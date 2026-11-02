//! Clean-room Attack Shark X6 passive battery-heartbeat adapter.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "attack-shark-x6";
pub const VID: u16 = 0x1d57;
pub const PID: u16 = 0xfa60;
pub const REPORT_LEN: usize = 5;

pub fn parse_heartbeat(report: &[u8]) -> Option<u8> {
    if report.len() != REPORT_LEN || report[..3] != [0x03, 0x10, 0x40] {
        return None;
    }
    let level = report[4];
    (1..=10).contains(&level).then_some(level * 10)
}

pub fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|interface| {
            interface.vid == VID
                && interface.pid == PID
                && interface.usage_page == 0x01
                && interface.usage == 0x80
                && interface.input_len as usize == REPORT_LEN
        })
        .collect();
    let [interface] = interfaces.as_slice() else {
        return Err("未找到唯一的 Attack Shark X6 接收器状态接口".into());
    };

    let mut transport = WinTransport::open(&interface.path)?;
    let deadline = Instant::now() + Duration::from_millis(1800);
    while Instant::now() < deadline {
        let report = transport.read(REPORT_LEN, Duration::from_millis(200))?;
        if let Some(percent) = parse_heartbeat(&report) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                battery_raw: Some(percent),
                connection_path: Some("2.4 GHz 接收器".into()),
                charging: None,
                wired: false,
                identity: "Attack Shark X6 · 1D57:FA60 被动电量心跳".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "只读取接收器推送的电量；充电状态未知。".into(),
                ..State::default()
            });
        }
    }
    Err("Attack Shark X6 接收器尚未推送有效电量心跳".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_heartbeat_tenths_to_percent() {
        assert_eq!(parse_heartbeat(&[0x03, 0x10, 0x40, 1, 1]), Some(10));
        assert_eq!(parse_heartbeat(&[0x03, 0x10, 0x40, 1, 10]), Some(100));
    }

    #[test]
    fn rejects_other_events_and_unknown_levels() {
        assert_eq!(parse_heartbeat(&[0x03, 0x10, 0x40, 1, 0]), None);
        assert_eq!(parse_heartbeat(&[0x03, 0x10, 0x40, 1, 11]), None);
        assert_eq!(parse_heartbeat(&[0x03, 0x10, 0x10, 1, 10]), None);
        assert_eq!(parse_heartbeat(&[0x03, 0x10, 0x40, 1]), None);
    }
}
