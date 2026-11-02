//! Clean-room AULA F99 Pro receiver identity and battery protocol adapter.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "aula-f99-pro";
pub const VID: u16 = 0x3554;
pub const PID: u16 = 0xfa09;
pub const REPORT_LEN: usize = 20;
pub const REPORT_ID: u8 = 0x13;
const PSD: [u8; 6] = [0x03, 0, 0, 0, 0x01, 0x58];

fn make_request(command: u8) -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[0] = REPORT_ID;
    frame[1] = command;
    frame
}

fn checksum_ok(frame: &[u8]) -> bool {
    frame.len() == REPORT_LEN
        && frame[19]
            == frame[..19]
                .iter()
                .fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

fn valid_single_page(frame: &[u8], command: u8, min_payload: u8) -> bool {
    checksum_ok(frame)
        && frame[0] == REPORT_ID
        && frame[1] == command
        && frame[2] == 1
        && frame[3] == 0
        && frame[4] >= min_payload
}

pub fn parse_identity(frame: &[u8]) -> bool {
    valid_single_page(frame, 0x05, 7) && frame[5..11] == PSD && frame[11] == 1
}

pub fn parse_battery(frame: &[u8]) -> Option<u8> {
    if !valid_single_page(frame, 0x0a, 4) || frame[7] != 1 {
        return None;
    }
    let percent = frame[6];
    (0..=100).contains(&percent).then_some(percent)
}

fn query(transport: &mut WinTransport, command: u8) -> Result<Vec<u8>, String> {
    let response_command = if command == 0x0b { 0x0a } else { command };
    transport.flush()?;
    transport.write(&make_request(command), Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(750);
    while Instant::now() < deadline {
        let reply = transport.read(REPORT_LEN, Duration::from_millis(100))?;
        if reply.len() == REPORT_LEN && reply[0] == REPORT_ID && reply[1] == response_command {
            return Ok(reply);
        }
    }
    Err("AULA F99 Pro 接收器查询超时".into())
}

pub fn read_state() -> Result<State, String> {
    let matches: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|interface| {
            interface.vid == VID
                && interface.pid == PID
                && interface.usage_page == 0xff02
                && interface.usage == 0x0002
                && interface.input_len as usize == REPORT_LEN
                && interface.output_len as usize == REPORT_LEN
        })
        .collect();
    let [interface] = matches.as_slice() else {
        return Err("未找到唯一的 AULA 2.4 GHz 接收器通道".into());
    };

    let mut transport = WinTransport::open(&interface.path)?;
    let identity = query(&mut transport, 0x05)?;
    if !parse_identity(&identity) {
        return Err("共享接收器返回的设备身份不是 AULA F99 Pro".into());
    }
    let battery = query(&mut transport, 0x0b)?;
    let percent = parse_battery(&battery).ok_or("AULA F99 Pro 返回了无效电量或未知连接状态")?;

    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        battery_raw: Some(percent),
        connection_path: Some("2.4 GHz 共用接收器".into()),
        charging: None,
        wired: false,
        identity: "AULA F99 Pro · 2.4 GHz 接收器 · PSD 已确认".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "只读取已确认身份的接收器电量；充电状态未知。".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(command: u8, payload: &[u8]) -> [u8; REPORT_LEN] {
        let mut frame = [0; REPORT_LEN];
        frame[0] = REPORT_ID;
        frame[1] = command;
        frame[2] = 1;
        frame[4] = payload.len() as u8;
        frame[5..5 + payload.len()].copy_from_slice(payload);
        frame[19] = frame[..19]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        frame
    }

    #[test]
    fn requires_f99_pro_identity_and_valid_battery_response() {
        let identity = reply(0x05, &[0x03, 0, 0, 0, 1, 0x58, 1, 0, 0, 0]);
        assert!(parse_identity(&identity));
        let battery = reply(0x0a, &[5, 0x64, 1, 0]);
        assert_eq!(parse_battery(&battery), Some(100));
    }

    #[test]
    fn rejects_unknown_identity_bad_checksum_and_out_of_range_battery() {
        let mut identity = reply(0x05, &[0x03, 0, 0, 0, 1, 0x58, 1, 0, 0, 0]);
        identity[10] = 0x59;
        identity[19] = identity[..19]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        assert!(!parse_identity(&identity));

        let mut battery = reply(0x0a, &[5, 101, 1, 0]);
        assert_eq!(parse_battery(&battery), None);
        let zero = reply(0x0a, &[5, 0, 1, 0]);
        assert_eq!(parse_battery(&zero), Some(0));
        battery[19] ^= 1;
        assert_eq!(parse_battery(&battery), None);
    }
}
