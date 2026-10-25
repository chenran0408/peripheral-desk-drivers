//! MCHOSE V9 Pro receiver battery query.
//! Adapted from Vigil Watt v1.3.0's MchoseV9ProCodec.cs and
//! MchosePlugin.QueryHeadsetV9 (a78e973b4eb634bdce37c9078dd48d1c07e3eb73).
//! Copyright (c) 2026 evren; the MIT license is included in this crate and
//! in the installed plugin archive as THIRD_PARTY_LICENSES/Vigil-Watt-MIT.txt.
use peripheral_desk::hid::{self, Interface};
use peripheral_desk::model::State;
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant};

pub const DEVICE_KEY: &str = "mchose-v9-pro";
pub const VID: u16 = 0x291d;
pub const PID: u16 = 0x385d;
const REPORT_ID: u8 = 0x55;
const REPORT_LEN: usize = 64;
const USAGE_PAGE: u16 = 0xff90;
const USAGE: u16 = 1;
const RESPONSE_WINDOW: Duration = Duration::from_millis(750);

pub fn query() -> [u8; REPORT_LEN] {
    let mut report = [0; REPORT_LEN];
    report[0] = REPORT_ID;
    report[1] = 0x65;
    report[2] = 0x01;
    report
}

/// The raw percentage is a single byte at response offset 2; no scaling.
/// 0 and 1 are rejected: upstream has only one 100% sample and notes 0 is the
/// receiver's sleep/off placeholder. Values above 100 are unknown formats.
pub fn parse_battery(report: &[u8]) -> Result<u8, &'static str> {
    if report.len() != REPORT_LEN || report[0] != REPORT_ID || report[1] != 0x65 {
        return Err("V9 Pro 回包长度或命令不匹配");
    }
    if report == query() {
        return Err("V9 Pro 收到查询帧回显");
    }
    match report[2] {
        0 => Err("V9 Pro 接收器返回休眠/关机占位值"),
        1 => Err("V9 Pro 1% 尚无可靠样本，按未知值处理"),
        2..=100 => Ok(report[2]),
        _ => Err("V9 Pro 电量字段超出 0–100"),
    }
}

fn supported(interface: &Interface) -> bool {
    interface.vid == VID
        && interface.pid == PID
        && interface
            .path
            .to_ascii_lowercase()
            .contains("&mi_00&col05#")
        && interface.usage_page == USAGE_PAGE
        && interface.usage == USAGE
        && interface.input_len as usize == REPORT_LEN
        && interface.output_len as usize == REPORT_LEN
        && interface.feature_len == 0
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|interface| supported(interface));
    let Some(interface) = matches.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            present: false,
            error: "未找到精确匹配的 MCHOSE V9 Pro 2.4 GHz 电量接口".into(),
            settings_error: "仅提供只读电量查询；连接方式为接收器。充电状态未确认。".into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err("找到多个 MCHOSE V9 Pro 电量接口，为避免查询错误通道已停止".into());
    }

    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;
    let request = query();
    transport.write(&request, Duration::from_millis(500))?;

    let deadline = Instant::now() + RESPONSE_WINDOW;
    let mut last_error = "V9 Pro 在 750 毫秒内没有有效电量回复".to_string();
    while Instant::now() < deadline {
        let reply = transport.read(
            REPORT_LEN,
            deadline.saturating_duration_since(Instant::now()),
        )?;
        if reply.is_empty() {
            break;
        }
        match parse_battery(&reply) {
            Ok(percent) => {
                return Ok(State {
                    key: DEVICE_KEY.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: None,
                    wired: false,
                    updated: now(),
                    identity: "MCHOSE V9 Pro · 2.4 GHz 接收器 · 291D:385D · report 55".into(),
                    settings_error:
                        "仅提供只读电量；充电状态未知，非满电电量映射未由上游样本确认。".into(),
                    ..State::default()
                });
            }
            Err(error) => last_error = error.into(),
        }
    }
    Err(last_error)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(percent: u8, status: u8) -> [u8; REPORT_LEN] {
        let mut report = [0; REPORT_LEN];
        report[0] = REPORT_ID;
        report[1] = 0x65;
        report[2] = percent;
        report[3] = status;
        report
    }

    #[test]
    fn query_and_upstream_single_point_response_are_exact() {
        let query = query();
        assert_eq!(&query[..4], &[0x55, 0x65, 1, 0]);
        assert_eq!(query.len(), 64);
        assert_eq!(parse_battery(&response(100, 2)), Ok(100));
    }

    #[test]
    fn malformed_echo_sleep_and_unknown_values_are_not_percentages() {
        assert!(parse_battery(&query()).is_err());
        assert!(parse_battery(&response(0, 2)).is_err());
        assert!(parse_battery(&response(1, 2)).is_err());
        assert!(parse_battery(&response(101, 2)).is_err());
        assert!(parse_battery(&response(55, 2)[..63]).is_err());
        let mut other_command = response(55, 2);
        other_command[1] = 0x64;
        assert!(parse_battery(&other_command).is_err());
    }
}
