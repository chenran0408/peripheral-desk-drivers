//! Read the classic Flydigi device-info report on the PC-mode DInput interface.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MODEL: &str = "flydigi-vader-4-pro";
const INFO_COMMAND: u8 = 0xec;
const INPUT_LENGTH: usize = 32;

#[derive(Debug, PartialEq, Eq)]
struct Reading {
    raw: u8,
    percent: Option<u8>,
    charging: Option<bool>,
    wired: Option<bool>,
}

fn parse_info(report: &[u8]) -> Option<Reading> {
    if report.len() != INPUT_LENGTH
        || report[0] != 0x04
        || report[15] != INFO_COMMAND
        || !matches!(report[3], 85 | 91 | 105)
    {
        return None;
    }
    let raw = report[11];
    let (percent, charging) = match (raw >> 4, raw & 0x0f) {
        (0, level @ 1..=5) => (Some(20 * level), Some(false)),
        (1, level @ 0..=5) => (Some(20 * level), Some(true)),
        (1, _) => (None, Some(true)),
        (2, _) => (Some(100), Some(false)),
        // A freshly connected dongle may answer before it knows the level.
        _ => (None, None),
    };
    let wired = match report[13] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Some(Reading {
        raw,
        percent,
        charging,
        wired,
    })
}

fn endpoint(i: &hid::Interface) -> bool {
    i.vid == 0x04b4
        && matches!(i.pid, 0x2412 | 0x2414)
        && i.path.to_ascii_lowercase().contains("&mi_02")
        && usize::from(i.input_len) == INPUT_LENGTH
        && (12..=64).contains(&i.output_len)
}

fn query(i: &hid::Interface) -> Result<Reading, String> {
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    // Windows WriteFile takes a buffer padded to OutputReportByteLength.
    let mut command = vec![0; usize::from(i.output_len)];
    command[..2].copy_from_slice(&[0x05, INFO_COMMAND]);
    io.write(&command, Duration::from_millis(250))?;
    let deadline = Instant::now() + Duration::from_millis(550);
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let report = io.read(INPUT_LENGTH, left)?;
        if report.is_empty() {
            break;
        }
        if let Some(reading) = parse_info(&report) {
            return Ok(reading);
        }
    }
    Err("未收到黑武士 4 Pro 的电量回包。请唤醒手柄并确认 PC 档已切到蓝灯 DInput。".into())
}

fn read_state(request: &Value) -> Result<State, String> {
    let target = hid::request_instance(request)?;
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces
        .iter()
        .filter(|i| endpoint(i))
        .map(|i| (i, i.instance_id()))
        .filter(|(_, id)| target.is_none_or(|target| target == id))
        .collect();
    let (_, instance_id) = candidates
        .first()
        .ok_or("未找到黑武士 4 Pro 的 DInput 接口。请在 PC 档长按 O＋A 切到蓝灯。")?;
    if candidates.iter().any(|(_, id)| id != instance_id) {
        return Err("检测到多个飞智接口，请选择要查看的手柄。".into());
    }
    let mut error = String::new();
    for (i, instance_id) in candidates {
        match query(i) {
            Ok(reading) => {
                let connection = match reading.wired {
                    Some(true) => "DInput · USB 有线",
                    Some(false) => "DInput · 2.4 GHz 接收器",
                    None => "DInput",
                };
                return Ok(State {
                    key: MODEL.into(),
                    instance_id,
                    present: true,
                    fresh: true,
                    percent: reading.percent,
                    battery_raw: Some(reading.raw),
                    connection_path: Some(connection.into()),
                    charging: reading.charging,
                    wired: reading.wired == Some(true),
                    identity: format!("Flydigi Vader 4 Pro · {:04X}:{:04X}", i.vid, i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    ..State::default()
                });
            }
            Err(failure) => error = failure,
        }
    }
    Err(error)
}

fn handle(key: &str, input: &str) -> String {
    let result = (|| {
        let request: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
        if key != MODEL || request["op"].as_str() != Some("read-state") {
            return Err("此插件只读取黑武士 4 Pro 的电量。".into());
        }
        read_state(&request)
    })();
    match result {
        Ok(state) => json!({"ok": true, "value": state, "error": ""}),
        Err(error) => json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("flydigi-vader-4-pro", handle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_vader_4_pro_info_reports_supply_a_battery_tier() {
        let mut report = [0; 32];
        report[0] = 0x04;
        report[3] = 85;
        report[11] = 2;
        report[15] = INFO_COMMAND;
        let reading = parse_info(&report).unwrap();
        assert_eq!(
            (reading.percent, reading.charging, reading.wired),
            (Some(40), Some(false), Some(false))
        );
        report[11] = 0;
        assert_eq!(parse_info(&report).unwrap().percent, None);
        report[11] = 0x13;
        report[13] = 1;
        let reading = parse_info(&report).unwrap();
        assert_eq!(
            (reading.percent, reading.charging, reading.wired),
            (Some(60), Some(true), Some(true))
        );
        report[11] = 0x20;
        let reading = parse_info(&report).unwrap();
        assert_eq!(
            (reading.percent, reading.charging),
            (Some(100), Some(false))
        );
        report[3] = 80; // Vader 3 Pro shares the family protocol.
        assert!(parse_info(&report).is_none());
        report[3] = 85;
        report[15] = 0;
        assert!(parse_info(&report).is_none());
    }
}
