use super::{Connection, Result, Transport, PAYLOAD_LEN, REPORT_ID};
use peripheral_desk::hid::{self, Interface};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant};

const USAGE_PAGE: u16 = 0xff02;
const REPORT_BUFFER_LEN: usize = PAYLOAD_LEN + 1;
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(700);

fn protocol_interface(interface: &Interface) -> bool {
    interface.vid == super::VID
        && Connection::from_pid(interface.pid).is_some()
        && interface.usage_page == USAGE_PAGE
        && interface.input_len as usize == REPORT_BUFFER_LEN
        && interface.output_len as usize == REPORT_BUFFER_LEN
}

fn conflicting_product_name(product: &str) -> bool {
    let product = product.to_ascii_lowercase();
    product.contains("f1 pro") || product.contains("f2 pro max")
}

struct VgnTransport(WinTransport);

impl Transport for VgnTransport {
    fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>> {
        self.0.flush()?;
        let mut wire = [0; REPORT_BUFFER_LEN];
        wire[0] = REPORT_ID;
        wire[1..].copy_from_slice(request);
        self.0.write(&wire, Duration::from_millis(200))?;

        let deadline = Instant::now() + RESPONSE_TIMEOUT;
        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let reply = self.0.read(REPORT_BUFFER_LEN, remaining)?;
            if reply.is_empty() {
                break;
            }
            if reply.len() != REPORT_BUFFER_LEN {
                return Err(format!(
                    "VGN 输入报告长度不匹配：期望 {REPORT_BUFFER_LEN}，收到 {}",
                    reply.len()
                ));
            }
            if reply[0] != REPORT_ID || reply[1] != request[0] {
                continue;
            }
            return Ok(reply[1..].to_vec());
        }
        Err(format!(
            "VGN 未在 {} ms 内回复 report 08 命令 0x{:02X}",
            RESPONSE_TIMEOUT.as_millis(),
            request[0]
        ))
    }
}

pub fn read_vgn() -> Result<peripheral_desk::model::State> {
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces
        .iter()
        .filter(|i| protocol_interface(i))
        .collect();
    match candidates.as_slice() {
        [] => Ok(peripheral_desk::model::State {
            key: super::DEVICE_KEY.into(),
            present: false,
            error: "未找到唯一的 VGN F2 Master+ report-08 集合（仅 3554:FB56/FB57、FF02、17 字节）".into(),
            settings_error: "仅提供只读电量、充电状态和公开固件版本；未开放设置写入。".into(),
            ..Default::default()
        }),
        [interface] => {
            if conflicting_product_name(&interface.product) {
                return Ok(peripheral_desk::model::State {
                    key: super::DEVICE_KEY.into(),
                    present: false,
                    error: "VGN 设备报告名称与 F2 Master+ 不符；未发送查询".into(),
                    settings_error: "仅提供只读电量、充电状态和公开固件版本；未开放设置写入。".into(),
                    ..Default::default()
                });
            }
            let connection = Connection::from_pid(interface.pid)
                .ok_or_else(|| "VGN 产品 ID 不属于 F2 Master+ 预览范围".to_string())?;
            let mut transport = VgnTransport(WinTransport::open(&interface.path)?);
            Ok(super::read_with(&mut transport, connection))
        }
        _ => Err("找到多个 VGN F2 Master+ report-08 集合；为避免查询错误集合，已停止".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interface(pid: u16) -> Interface {
        Interface {
            path: "fixture".into(),
            vid: super::super::VID,
            pid,
            usage_page: USAGE_PAGE,
            usage: 0,
            feature_len: 0,
            input_len: REPORT_BUFFER_LEN as u16,
            output_len: REPORT_BUFFER_LEN as u16,
            product: "Dragonfly F2 Master+".into(),
        }
    }

    #[test]
    fn only_exact_vgn_collection_and_report_shape_match() {
        let mut item = interface(super::super::RECEIVER_PID);
        assert!(protocol_interface(&item));
        item.pid = 0xfb3d;
        assert!(!protocol_interface(&item));
        item = interface(super::super::WIRED_PID);
        item.usage_page = 0xff01;
        assert!(!protocol_interface(&item));
        item = interface(super::super::WIRED_PID);
        item.input_len = 16;
        assert!(!protocol_interface(&item));
        item = interface(super::super::WIRED_PID);
        item.vid = 0x3555;
        assert!(!protocol_interface(&item));
    }

    #[test]
    fn conflicting_product_identity_is_not_queried() {
        assert!(!conflicting_product_name(""));
        assert!(!conflicting_product_name("Dragonfly F2 Master+"));
        assert!(conflicting_product_name("VGN F1 Pro"));
        assert!(conflicting_product_name("Dragonfly F2 Pro Max"));
    }
}
