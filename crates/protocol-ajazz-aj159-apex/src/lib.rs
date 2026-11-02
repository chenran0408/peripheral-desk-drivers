//! Clean-room AJAZZ AJ159 APEX 2.4 GHz battery-status adapter.
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "ajazz-aj159-apex";
pub const VID: u16 = 0x3151;
pub const PID: u16 = 0x5007;
pub const REPORT_LEN: usize = 65;
const STATUS_REPORT_ID: u8 = 0x05;
const STATUS_POLL: u8 = 0xf7;

pub fn parse_battery(report: &[u8]) -> Option<u8> {
    if report.len() != REPORT_LEN
        || report[0] != STATUS_REPORT_ID
        || report[1] != 0
        || report[2] != 0
    {
        return None;
    }
    (1..=100).contains(&report[3]).then_some(report[3])
}

pub fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|interface| {
            interface.vid == VID
                && interface.pid == PID
                && interface.usage_page == 0xffff
                && interface.usage == 2
                && interface.feature_len as usize == REPORT_LEN
        })
        .collect();
    let [interface] = interfaces.as_slice() else {
        return Err("未找到唯一的 AJAZZ AJ159 APEX 2.4 GHz 控制接口".into());
    };

    let device = hid::Device::open(&interface.path)?;
    let mut poll = [0; REPORT_LEN];
    poll[1] = STATUS_POLL;
    device.send(&poll)?;
    std::thread::sleep(Duration::from_millis(30));
    let report = device.receive(STATUS_REPORT_ID, REPORT_LEN)?;
    let percent = parse_battery(&report).ok_or("AJAZZ 接收器尚未报告有效电量")?;

    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        battery_raw: Some(percent),
        connection_path: Some("AJAZZ 2.4G 8K 接收器".into()),
        charging: None,
        wired: false,
        identity: "AJAZZ AJ159 APEX (2.4G 8K) · 3151:5007".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅读取接收器电量；充电状态未知。".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_a_valid_reported_charge() {
        let mut report = [0; REPORT_LEN];
        report[0] = STATUS_REPORT_ID;
        report[3] = 62;
        assert_eq!(parse_battery(&report), Some(62));
        report[3] = 0;
        assert_eq!(parse_battery(&report), None);
        report[1] = 1;
        report[3] = 62;
        assert_eq!(parse_battery(&report), None);
        report[1] = 0;
        report[3] = 101;
        assert_eq!(parse_battery(&report), None);
    }

    #[test]
    fn rejects_wrong_report_and_length() {
        let mut report = [0; REPORT_LEN];
        report[0] = 0x04;
        report[3] = 62;
        assert_eq!(parse_battery(&report), None);
        report[0] = STATUS_REPORT_ID;
        assert_eq!(parse_battery(&report[..REPORT_LEN - 1]), None);
    }
}
