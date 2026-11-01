//! Shared read-only adapter for Razer wireless receiver PIDs listed by Voltide.
//!
//! The transport is the 91-byte Windows Feature report used by Voltide's
//! Razer provider. Model selection stays in each wrapper crate: this module
//! only supplies the exact PID guard and the battery query.

use peripheral_desk::{
    hid::{self, Device, Interface},
    model::State,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VID: u16 = 0x1532;
const REPORT_LEN: usize = 91;
const BATTERY_CLASS: u8 = 0x07;
const BATTERY_COMMAND: u8 = 0x80;
const TRANSACTION_ID: u8 = 0xee;

fn supported(interface: &Interface, pid: u16) -> bool {
    interface.vid == VID && interface.pid == pid && interface.feature_len == REPORT_LEN as u16
}

fn checksum(report: &[u8; REPORT_LEN]) -> u8 {
    report[3..89].iter().fold(0, |sum, byte| sum ^ byte)
}

fn request() -> [u8; REPORT_LEN] {
    let mut report = [0; REPORT_LEN];
    report[2] = TRANSACTION_ID;
    report[6] = 2;
    report[7] = BATTERY_CLASS;
    report[8] = BATTERY_COMMAND;
    report[9] = 0;
    report[10] = 0;
    report[89] = checksum(&report);
    report
}

fn battery_from_reply(reply: &[u8]) -> Result<u8, String> {
    let reply = <&[u8; REPORT_LEN]>::try_from(reply)
        .map_err(|_| "Razer Feature 回复长度不匹配".to_string())?;
    if reply[0] != 0
        || reply[2] != TRANSACTION_ID
        || reply[7] != BATTERY_CLASS
        || reply[8] != BATTERY_COMMAND
        || reply[89] != checksum(reply)
    {
        return Err("Razer Feature 回复校验或命令匹配失败".into());
    }
    match reply[1] {
        0x02 if reply[6] >= 2 => Ok(reply[10].min(100)),
        0x04 => Err("Razer 接收器未响应，设备可能处于休眠状态".into()),
        status => Err(format!("Razer 电量查询失败（状态 0x{status:02X}）")),
    }
}

fn query(device: &Device) -> Result<u8, String> {
    let report = request();
    let mut last_error = String::from("Razer 接收器没有返回电量");
    for attempt in 0..4 {
        device.send(&report)?;
        std::thread::sleep(Duration::from_millis(35));
        match battery_from_reply(&device.receive(0, REPORT_LEN)?) {
            Ok(value) => return Ok(value),
            Err(error) => last_error = error,
        }
        if attempt < 3 {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    Err(last_error)
}

/// Read battery state for one exact Razer receiver PID.
pub fn read_state(key: &str, model: &str, pid: u16) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matches = interfaces
        .iter()
        .filter(|interface| supported(interface, pid));
    let Some(interface) = matches.next() else {
        return Ok(State {
            key: key.into(),
            error: format!("未找到 Razer {model} 接收器（1532:{pid:04X}）"),
            settings_error: "仅支持 Voltide 已确认的 91 字节 Feature 电量查询；未适配充电、DPI、灯效、改键或宏。".into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err(format!("发现多个 Razer {model} 匹配接口，已停止查询"));
    }
    let device = Device::open(&interface.path)?;
    let percent = query(&device)?;
    let updated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(State {
        key: key.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        battery_raw: Some(percent),
        wired: false,
        updated,
        identity: format!("Razer {model} · 1532:{pid:04X} · 91-byte Feature"),
        settings_error:
            "仅支持 Voltide 已确认的 91 字节 Feature 电量查询；未适配充电、DPI、灯效、改键或宏。"
                .into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_request_has_voltide_frame_shape() {
        let report = request();
        assert_eq!(report[2], TRANSACTION_ID);
        assert_eq!(&report[6..11], &[2, BATTERY_CLASS, BATTERY_COMMAND, 0, 0]);
        assert_eq!(report[89], checksum(&report));
    }

    #[test]
    fn parses_percent_and_rejects_wrong_transaction() {
        let mut reply = request();
        reply[1] = 2;
        reply[6] = 2;
        reply[10] = 73;
        reply[89] = checksum(&reply);
        assert_eq!(battery_from_reply(&reply), Ok(73));
        reply[2] = 0;
        reply[89] = checksum(&reply);
        assert!(battery_from_reply(&reply).is_err());
    }
}
