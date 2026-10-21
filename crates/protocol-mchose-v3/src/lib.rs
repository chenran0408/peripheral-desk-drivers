//! Read-only A7 V3 Ultra+ receiver adapter.
//!
//! Protocol facts: OpenMouse mouse-protocol bac031567b28f2a55c616fc525b03f2d5fa452cc,
//! src/mchose/v3.ts, src/drivers/mchose/v3-hid.ts and docs/mchose-protocol.md.
//! This Rust codec is independently authored; no upstream implementation code is copied.
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const VID: u16 = 0x3837;
const RECEIVER_PID: u16 = 0x1018;
const USAGE_PAGE: u16 = 0xff01;
const USAGE: u16 = 0x0001;
const REPORT_ID: u8 = 0x4d;
const REPORT_LEN: usize = 64;
const BODY_LEN: usize = REPORT_LEN - 1;
const INFO_COMMAND: u16 = 0x0900;
const PRODUCT_NAME: &str = "MCHOSE A7 V3 Ultra+";
pub const DEVICE_KEY: &str = "mchose-a7-v3-ultra-plus";

type Result<T> = std::result::Result<T, String>;

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |sum, byte| sum ^ byte)
}

fn request() -> [u8; REPORT_LEN] {
    let mut wire = [0; REPORT_LEN];
    wire[0] = REPORT_ID;
    let body = &mut wire[1..];
    body[0] = 1;
    body[1] = 1;
    body[2] = 0;
    body[3..5].copy_from_slice(&INFO_COMMAND.to_le_bytes());
    body[5] = 0;
    body[6] = 0;
    body[7] = checksum(&body[1..7]);
    wire
}

fn reply_info(wire: &[u8], request: &[u8; REPORT_LEN]) -> Result<[u8; 16]> {
    if wire.len() != BODY_LEN + 1 || wire[0] != REPORT_ID {
        return Err("MCHOSE 输入报告长度或 Report ID 不匹配".into());
    }
    let body = &wire[1..];
    if body[0] != 1 || body[1] != 1 {
        return Err("MCHOSE 回复协议版本或校验标志不匹配".into());
    }
    let len = body[2] as usize;
    if body[3..5] != INFO_COMMAND.to_le_bytes() {
        return Err("MCHOSE 回复命令不匹配".into());
    }
    if body[5] != request[6] || body[6] != request[7] {
        return Err("MCHOSE 回复事务标识不匹配".into());
    }
    if len > BODY_LEN - 8 || body[7 + len] != checksum(&body[1..7 + len]) {
        return Err("MCHOSE 回复校验不匹配".into());
    }
    if len == 1 && body[7] == 0xff {
        return Err("MCHOSE 接收器已响应，但鼠标当前不可达".into());
    }
    if len != 16 {
        return Err("MCHOSE 身份数据长度不匹配".into());
    }
    body[7..7 + len]
        .try_into()
        .map_err(|_| "MCHOSE 身份数据不完整".into())
}

fn verify_identity(data: &[u8; 16], product: &str) -> Result<(u8, bool, bool)> {
    let vendor = u16::from_le_bytes([data[0], data[1]]);
    let product_id = u16::from_le_bytes([data[2], data[3]]);
    if vendor != VID || product_id != 0x4026 || product != PRODUCT_NAME {
        return Err(format!(
            "MCHOSE 接收器后的型号身份不匹配：内部 {:04X}:{:04X}，产品名 {:?}",
            vendor, product_id, product
        ));
    }
    let charge = match data[11] {
        0 => false,
        1 => true,
        value => return Err(format!("未知 MCHOSE 充电状态 0x{value:02X}")),
    };
    if data[12] > 100 {
        return Err(format!("MCHOSE 电量超出 0–100：{}", data[12]));
    }
    let connected = match data[10] {
        0 => false,
        2 => true,
        value => return Err(format!("未知 MCHOSE 接收器连接状态 0x{value:02X}")),
    };
    Ok((data[12], charge, connected))
}

fn matching_interface(i: &peripheral_desk::hid::Interface) -> bool {
    i.vid == VID
        && i.pid == RECEIVER_PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.product == PRODUCT_NAME
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

pub fn read_state() -> Result<peripheral_desk::model::State> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = matches.next() else {
        return Ok(peripheral_desk::model::State {
            key: DEVICE_KEY.into(),
            error: "未找到精确的 MCHOSE A7 V3 Ultra+ 8K 接收器 HID 集合".into(),
            settings_error: "仅提供未实测的只读身份与电量预览；无设置写入。".into(),
            updated: now(),
            ..Default::default()
        });
    };
    if matches.next().is_some() {
        return Err("找到多个匹配的 MCHOSE 接收器集合；为避免读取错误设备已停止".into());
    }

    let mut transport = WinTransport::open(&interface.path)?;
    let request = request();
    let deadline = Instant::now() + Duration::from_millis(1500);
    transport.flush()?;
    transport.write(
        &request,
        deadline
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(500)),
    )?;
    let response = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("MCHOSE 0x0900 身份/电量查询超时".into());
        }
        let wire = transport.read(REPORT_LEN, remaining)?;
        if wire.is_empty() {
            continue;
        }
        if wire.len() == REPORT_LEN
            && wire[0] == REPORT_ID
            && wire[1] == request[1]
            && wire[4..6] == request[4..6]
        {
            break wire;
        }
    };
    let data = reply_info(&response, &request)?;
    let (percent, charging, connected) = verify_identity(&data, &interface.product)?;
    Ok(peripheral_desk::model::State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: connected,
        percent: connected.then_some(percent),
        charging: connected.then_some(charging),
        wired: false,
        updated: now(),
        identity: "MCHOSE A7 V3 Ultra+ · USB 3837:1018 receiver · identity 3837:4026".into(),
        error: if connected {
            String::new()
        } else {
            "MCHOSE 接收器已响应，但报告鼠标未连接；未显示电量".into()
        },
        settings_error:
            "仅开放只读身份、电量与充电状态；固件未知；设置、按键、宏和灯光未开放。未实机验证。"
                .into(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn captured_identity_reply(req: &[u8; REPORT_LEN]) -> [u8; REPORT_LEN] {
        // Identity/power payload from the fixed upstream A7 V3 Ultra+ receiver capture.
        let payload = [
            0x37, 0x38, 0x26, 0x40, 4, 0, 0, 0, 0, 0x10, 2, 1, 0x55, 0, 8, 0xe4,
        ];
        let mut wire = [0; REPORT_LEN];
        wire[0] = REPORT_ID;
        let body = &mut wire[1..];
        body[0] = 1;
        body[1] = 1;
        body[2] = payload.len() as u8;
        body[3..5].copy_from_slice(&INFO_COMMAND.to_le_bytes());
        body[5] = req[6];
        body[6] = req[7];
        body[7..7 + payload.len()].copy_from_slice(&payload);
        body[7 + payload.len()] = checksum(&body[1..7 + payload.len()]);
        wire
    }

    #[test]
    fn builds_only_the_zero_payload_identity_read() {
        let wire = request();
        assert_eq!(wire[0], REPORT_ID);
        assert_eq!(&wire[1..8], &[1, 1, 0, 0, 9, 0, 0]);
        assert_eq!(wire[8], checksum(&wire[2..8]));
    }

    #[test]
    fn validates_captured_identity_and_rejects_corrupt_or_mismatched_reply() {
        let req = request();
        let mut reply = captured_identity_reply(&req);
        let data = reply_info(&reply, &req).unwrap();
        assert_eq!(
            verify_identity(&data, PRODUCT_NAME).unwrap(),
            (0x55, true, true)
        );
        assert!(verify_identity(&data, "MCHOSE A5 V3 Ultra+").is_err());

        reply[1 + 20] ^= 1;
        assert!(reply_info(&reply, &req).is_err());
        let mut reply = captured_identity_reply(&req);
        reply[1 + 3] ^= 1;
        assert!(reply_info(&reply, &req).is_err());
    }

    #[test]
    fn refuses_unknown_charge_and_out_of_range_battery() {
        let mut data = [0; 16];
        data[..4].copy_from_slice(&[0x37, 0x38, 0x26, 0x40]);
        assert!(verify_identity(&data, PRODUCT_NAME).is_ok());
        data[11] = 2;
        assert!(verify_identity(&data, PRODUCT_NAME).is_err());
        data[11] = 0;
        data[12] = 101;
        assert!(verify_identity(&data, PRODUCT_NAME).is_err());
    }

    #[test]
    fn disconnected_and_unknown_connection_states_are_not_presented_as_fresh() {
        let req = request();
        let mut reply = captured_identity_reply(&req);
        reply[1 + 7 + 10] = 0;
        let checksum_at = 1 + 7 + 16;
        reply[checksum_at] = checksum(&reply[2..checksum_at]);
        let data = reply_info(&reply, &req).unwrap();
        assert_eq!(
            verify_identity(&data, PRODUCT_NAME).unwrap(),
            (0x55, true, false)
        );

        reply[1 + 7 + 10] = 3;
        reply[checksum_at] = checksum(&reply[2..checksum_at]);
        let data = reply_info(&reply, &req).unwrap();
        assert!(verify_identity(&data, PRODUCT_NAME).is_err());
    }

    #[test]
    fn rejects_the_upstream_unreachable_mouse_marker() {
        let req = request();
        let mut wire = [0; REPORT_LEN];
        wire[0] = REPORT_ID;
        let body = &mut wire[1..];
        body[..7].copy_from_slice(&[1, 1, 1, 0, 9, 0, 0]);
        body[7] = 0xff;
        body[8] = checksum(&body[1..8]);
        assert!(reply_info(&wire, &req).unwrap_err().contains("不可达"));
    }

    #[test]
    fn only_exact_receiver_collection_and_product_name_match() {
        let mut interface = peripheral_desk::hid::Interface {
            path: r"\\?\hid#vid_3837&pid_1018".into(),
            vid: VID,
            pid: RECEIVER_PID,
            usage_page: USAGE_PAGE,
            usage: USAGE,
            feature_len: 0,
            input_len: REPORT_LEN as u16,
            output_len: REPORT_LEN as u16,
            product: PRODUCT_NAME.into(),
        };
        assert!(matching_interface(&interface));
        interface.product.push(' ');
        assert!(!matching_interface(&interface));
        interface.product = PRODUCT_NAME.into();
        interface.pid = 0x1014;
        assert!(!matching_interface(&interface));
        interface.pid = RECEIVER_PID;
        interface.output_len = 63;
        assert!(!matching_interface(&interface));
    }
}
