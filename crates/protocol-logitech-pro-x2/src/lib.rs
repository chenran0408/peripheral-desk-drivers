//! Read-only Logitech PRO X 2 LIGHTSPEED receiver battery protocol.
//!
//! Adapted from Vigil Watt v1.3.0 `CenturionCodec.cs` and the exact matching
//! scope in `LogitechPlugin.cs` (commit a78e973b4eb634bdce37c9078dd48d1c07e3eb73).
//! Copyright (c) 2026 evren; MIT license is retained in this crate and
//! distributed with its plugin as `THIRD_PARTY_LICENSES/Vigil-Watt-MIT.txt`.
//! Charging-byte interpretation is limited to the upstream documented values.

use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "logitech-pro-x2-lightspeed";
pub const VID: u16 = 0x046d;
pub const PID: u16 = 0x0af7;
pub const USAGE_PAGE: u16 = 0xffa0;
pub const USAGE: u16 = 1;
pub const REPORT_LEN: usize = 64;
const REPORT_ID: u8 = 0x51;
const RESPONSE_WINDOW: Duration = Duration::from_millis(6_500);

/// Fixed legacy query documented and used by Vigil Watt. No command discovery.
pub fn battery_request() -> [u8; REPORT_LEN] {
    let mut packet = [0; REPORT_LEN];
    packet[0] = REPORT_ID;
    packet[1] = 0x08;
    packet[3] = 0x03;
    packet[4] = 0x1a;
    packet[6] = 0x03;
    packet[8] = 0x04;
    packet[9] = 0x0a;
    packet
}

/// Returns the raw integer percentage and only the charging states supported
/// by the upstream record. All other status-byte values remain unknown.
pub fn parse_battery(packet: &[u8]) -> Result<(u8, Option<bool>), &'static str> {
    if packet.len() < 13
        || packet[0] != REPORT_ID
        || packet[1] != 0x0b
        || packet[8] != 0x04
        || packet[10] > 100
    {
        return Err("PRO X 2 电池应答标记或长度不匹配");
    }
    let charging = match packet[12] {
        0x00 => Some(false),
        0x02 => Some(true),
        _ => None,
    };
    Ok((packet[10], charging))
}

fn supported(interface: &Interface) -> bool {
    interface.vid == VID
        && interface.pid == PID
        && interface.usage_page == USAGE_PAGE
        && interface.usage == USAGE
        && interface.input_len as usize >= REPORT_LEN
        && interface.output_len as usize >= REPORT_LEN
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|item| supported(item));
    let Some(interface) = matches.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            present: false,
            error: "未找到 PRO X 2 LIGHTSPEED 的精确接收器通道".into(),
            settings_error:
                "本插件只支持 046D:0AF7 的 2.4 GHz 接收器；蓝牙/USB 有线不在本条目范围内。".into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err("找到多个 PRO X 2 Centurion 通道，停止查询以免选择错误接口".into());
    }

    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;
    transport.write(&battery_request(), Duration::from_millis(500))?;
    let deadline = Instant::now() + RESPONSE_WINDOW;
    let mut last_error = "PRO X 2 在 6.5 秒内没有有效电量回复".to_string();
    while Instant::now() < deadline {
        let reply = transport.read(
            REPORT_LEN,
            deadline.saturating_duration_since(Instant::now()),
        )?;
        if reply.is_empty() {
            break;
        }
        match parse_battery(&reply) {
            Ok((percent, charging)) => {
                return Ok(State {
                    key: DEVICE_KEY.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging,
                    wired: false,
                    updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    identity: "Logitech PRO X 2 LIGHTSPEED · 2.4 GHz · 046D:0AF7 · FFA0:0001".into(),
                    settings_error: "Vigil Watt 只提供电量查询；本条插件未接入耳机控制、EQ 或音效写入。USB 有线模式不支持。".into(),
                    ..State::default()
                });
            }
            Err(error) => last_error = error.into(),
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(percent: u8, status: u8) -> [u8; REPORT_LEN] {
        let mut packet = [0; REPORT_LEN];
        packet[0] = REPORT_ID;
        packet[1] = 0x0b;
        packet[8] = 0x04;
        packet[10] = percent;
        packet[12] = status;
        packet
    }

    #[test]
    fn query_matches_vigil_watt_centurion_request() {
        assert_eq!(
            &battery_request()[..10],
            &[0x51, 0x08, 0, 0x03, 0x1a, 0, 0x03, 0, 0x04, 0x0a]
        );
    }

    #[test]
    fn parses_integer_percent_and_known_charging_states() {
        assert_eq!(parse_battery(&reply(45, 0)), Ok((45, Some(false))));
        assert_eq!(parse_battery(&reply(43, 2)), Ok((43, Some(true))));
        assert_eq!(parse_battery(&reply(100, 3)), Ok((100, None)));
    }

    #[test]
    fn rejects_wrong_identity_marker_and_out_of_range_values() {
        assert!(parse_battery(&reply(101, 0)).is_err());
        let mut malformed = reply(50, 0);
        malformed[1] = 0x08;
        assert!(parse_battery(&malformed).is_err());
        malformed = reply(50, 0);
        malformed[8] = 0;
        assert!(parse_battery(&malformed).is_err());
        assert!(parse_battery(&[0; 12]).is_err());
    }
}
