//! ATK N9 V2 Ultimate receiver battery status, independently ported from
//! Voltide v2.1.2 (73f16fd2be44570c0a861cef6b4804d743656893).
//! Upstream reports the exact receiver/protocol as contributor hardware verified;
//! Peripheral Desk has not independently tested this device.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "atk-n9-v2-ultimate";
const VID: u16 = 0x373b;
const PID: u16 = 0x1262;
const USAGE_PAGE: u16 = 0xff90;
const REPORT_LEN: usize = 64;
const REPORT_ID: u8 = 0x55;
const COMMAND: u8 = 0x65;
const TARGET_HEADSET: u8 = 0x01;
const CHARGE_DISCHARGING: u8 = 0x02;
const CHARGE_CHARGING: u8 = 0x03;
const CHARGE_FULL: u8 = 0x04;
const CHARGE_UNKNOWN: u8 = 0x1a;

fn matches(interface: &hid::Interface) -> bool {
    interface.vid == VID
        && interface.pid == PID
        && interface.usage_page == USAGE_PAGE
        && interface.input_len as usize == REPORT_LEN
        && interface.output_len as usize == REPORT_LEN
}

fn query() -> [u8; REPORT_LEN] {
    let mut report = [0; REPORT_LEN];
    report[0] = REPORT_ID;
    report[1] = COMMAND;
    report[2] = TARGET_HEADSET;
    report
}

fn parse(reply: &[u8]) -> Option<(u8, Option<bool>)> {
    if reply.len() < 4 || reply[0] != REPORT_ID || reply[1] != COMMAND || reply[2] > 100 {
        return None;
    }
    let percent = if reply[3] == CHARGE_FULL {
        100
    } else {
        reply[2]
    };
    let charging = match reply[3] {
        CHARGE_DISCHARGING | CHARGE_FULL => Some(false),
        CHARGE_CHARGING => Some(true),
        CHARGE_UNKNOWN => None,
        _ => return None,
    };
    Some((percent, charging))
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|interface| matches(interface));
    let Some(interface) = matches.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 ATK N9 V2 Ultimate 指定接收器电量接口".into(),
            settings_error:
                "仅支持 373B:1262 接收器的只读电量与充电状态；其他连接方式及控制功能未适配。".into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err("发现多个 ATK N9 V2 Ultimate 电量接口，已停止查询".into());
    }

    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;
    transport.write(&query(), Duration::from_millis(200))?;

    let deadline = Instant::now() + Duration::from_millis(1200);
    let (percent, charging) = loop {
        if Instant::now() >= deadline {
            return Err("ATK N9 V2 Ultimate 电量查询未收到有效接收器回复".into());
        }
        let reply = transport.read(REPORT_LEN, Duration::from_millis(100))?;
        if let Some(value) = parse(&reply) {
            break value;
        }
    };

    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        battery_raw: Some(percent),
        connection_path: Some("ATK 2.4 GHz 接收器 373B:1262 / FF90 中断输出".into()),
        charging,
        wired: false,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: "ATK N9 V2 Ultimate · 2.4 GHz 接收器 373B:1262 · FF90 · 64 字节中断输出".into(),
        settings_error: "只读电量与充电状态；EQ、虚拟 7.1、灯效和其他控制功能未集成。".into(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("ATK N9 V2 Ultimate 插件仅提供接收器电量与充电状态读取".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_known_battery_responses_and_preserves_unknown_charge_state() {
        let mut reply = [0; REPORT_LEN];
        reply[..4].copy_from_slice(&[REPORT_ID, COMMAND, 47, CHARGE_DISCHARGING]);
        assert_eq!(parse(&reply), Some((47, Some(false))));
        reply[3] = CHARGE_CHARGING;
        assert_eq!(parse(&reply), Some((47, Some(true))));
        reply[3] = CHARGE_FULL;
        reply[2] = 93;
        assert_eq!(parse(&reply), Some((100, Some(false))));
        reply[3] = CHARGE_UNKNOWN;
        assert_eq!(parse(&reply), Some((93, None)));
        reply[3] = 0xff;
        assert_eq!(parse(&reply), None);
        reply[2] = 101;
        reply[3] = CHARGE_DISCHARGING;
        assert_eq!(parse(&reply), None);
    }
}

peripheral_desk_driver_ffi::export_driver!("atk-n9-v2-ultimate", handler);
