use super::{
    Connection, DeviceSpec, Result, Transport, F1_PRO, F2_MASTER_PLUS, F2_PRO_MAX, PAYLOAD_LEN,
    REPORT_ID,
};
use peripheral_desk::hid::{self, Interface};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant};

const USAGE_PAGE: u16 = 0xff02;
const REPORT_BUFFER_LEN: usize = PAYLOAD_LEN + 1;
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(700);

fn protocol_interface(interface: &Interface, device: DeviceSpec) -> bool {
    interface.vid == super::VID
        && Connection::from_pid_for(interface.pid, device).is_some()
        && interface.usage_page == USAGE_PAGE
        && device
            .required_usage
            .is_none_or(|usage| interface.usage == usage)
        && interface.input_len as usize == REPORT_BUFFER_LEN
        && interface.output_len as usize == REPORT_BUFFER_LEN
}

fn conflicting_product_name(product: &str, device: DeviceSpec) -> bool {
    let product = product.to_ascii_lowercase();
    let names = [F1_PRO, F2_PRO_MAX, F2_MASTER_PLUS];
    names.iter().any(|other| {
        other.key != device.key
            && match other {
                model if model.key == F1_PRO.key => product.contains("f1 pro"),
                model if model.key == F2_PRO_MAX.key => product.contains("f2 pro max"),
                _ => product.contains("f2 master+"),
            }
    })
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
    read_vgn_for(F2_MASTER_PLUS)
}

pub fn read_vgn_for(device: DeviceSpec) -> Result<peripheral_desk::model::State> {
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces
        .iter()
        .filter(|i| protocol_interface(i, device))
        .collect();
    match candidates.as_slice() {
        [] => Ok(peripheral_desk::model::State {
            key: device.key.into(),
            present: false,
            error: format!(
                "未找到唯一的 {} report-08 集合（仅 3554:{:04X}/{:04X}、FF02、17 字节）",
                device.name, device.receiver_pid, device.wired_pid
            ),
            settings_error: if device.reads_firmware {
                "仅提供只读电量、充电状态和公开固件版本；未开放设置写入。"
            } else {
                "仅提供只读电量与充电状态；该型号未核对固件读取命令，未开放设置写入。"
            }
            .into(),
            ..Default::default()
        }),
        [interface] => {
            if conflicting_product_name(&interface.product, device) {
                return Ok(peripheral_desk::model::State {
                    key: device.key.into(),
                    present: false,
                    error: format!("VGN 设备报告名称与 {} 不符；未发送查询", device.name),
                    settings_error: if device.reads_firmware {
                        "仅提供只读电量、充电状态和公开固件版本；未开放设置写入。"
                    } else {
                        "仅提供只读电量与充电状态；该型号未核对固件读取命令，未开放设置写入。"
                    }
                    .into(),
                    ..Default::default()
                });
            }
            let connection = Connection::from_pid_for(interface.pid, device)
                .ok_or_else(|| format!("VGN 产品 ID 不属于 {} 预览范围", device.name))?;
            let mut transport = VgnTransport(WinTransport::open(&interface.path)?);
            Ok(super::read_with_model(&mut transport, connection, device))
        }
        _ => Err(format!(
            "找到多个 VGN {} report-08 集合；为避免查询错误集合，已停止",
            device.name
        )),
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
        assert!(protocol_interface(&item, F2_MASTER_PLUS));
        item.pid = 0xfb3d;
        assert!(!protocol_interface(&item, F2_MASTER_PLUS));
        item = interface(super::super::WIRED_PID);
        item.usage_page = 0xff01;
        assert!(!protocol_interface(&item, F2_MASTER_PLUS));
        item = interface(super::super::WIRED_PID);
        item.input_len = 16;
        assert!(!protocol_interface(&item, F2_MASTER_PLUS));
        item = interface(super::super::WIRED_PID);
        item.vid = 0x3555;
        assert!(!protocol_interface(&item, F2_MASTER_PLUS));
    }

    #[test]
    fn conflicting_product_identity_is_not_queried() {
        assert!(!conflicting_product_name("", F2_MASTER_PLUS));
        assert!(!conflicting_product_name(
            "Dragonfly F2 Master+",
            F2_MASTER_PLUS
        ));
        assert!(conflicting_product_name("VGN F1 Pro", F2_MASTER_PLUS));
        assert!(conflicting_product_name(
            "Dragonfly F2 Pro Max",
            F2_MASTER_PLUS
        ));
        assert!(conflicting_product_name("Dragonfly F2 Master+", F2_PRO_MAX));
        assert!(!conflicting_product_name("VGN F2 Pro Max", F2_PRO_MAX));
        assert!(conflicting_product_name("VGN F2 Pro Max", F1_PRO));
        assert!(!conflicting_product_name("VGN F1 Pro", F1_PRO));
    }

    #[test]
    fn new_vgn_profiles_require_the_vendor_collection_usage() {
        for (device, pid) in [(F1_PRO, 0xf503), (F2_PRO_MAX, 0xfb3e)] {
            let mut item = interface(pid);
            item.usage = 2;
            assert!(protocol_interface(&item, device));
            item.usage = 0;
            assert!(!protocol_interface(&item, device));
        }
    }
}
