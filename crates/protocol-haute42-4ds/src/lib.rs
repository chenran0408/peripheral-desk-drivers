//! Read-only Haute42 Adapter 4DS identity and battery reports.
//!
//! Adapted from Vigil Watt v1.3.0 `HauteCodec.cs` / `HautePlugin.cs`, commit
//! a78e973b4eb634bdce37c9078dd48d1c07e3eb73. Copyright (c) 2026 evren;
//! MIT license is retained in this crate and plugin archive.

use peripheral_desk::{
    hid::{self, Device, Interface},
    model::State,
};
use std::{
    string::String,
    time::{SystemTime, UNIX_EPOCH},
};

pub const DEVICE_KEY: &str = "haute42-adapter-4ds";
pub const VID: u16 = 0x054c;
pub const PID: u16 = 0x0ce6;
const USAGE_PAGE: u16 = 1;
const USAGE: u16 = 5;
const FIRMWARE: u8 = 0xf8;
const STATUS: u8 = 0xf9;
const DEVICE_INFO: u8 = 0xfa;

pub fn parse_firmware(frame: &[u8]) -> bool {
    frame.len() >= 9
        && frame[0] == FIRMWARE
        && String::from_utf8_lossy(&frame[1..]).starts_with("Haute42 ")
}

pub fn parse_chip_id(frame: &[u8]) -> Option<String> {
    if frame.len() < 9 || frame[0] != DEVICE_INFO {
        return None;
    }
    let bytes = &frame[1..9];
    if bytes.iter().all(|b| *b == 0) || bytes.iter().all(|b| *b == 0xff) {
        return None;
    }
    Some(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// `None` percent is an explicit upstream no-reading marker, not zero percent.
/// The core's boolean charging field cannot represent the adapter's `2=full`,
/// so that state remains unknown instead of being mislabeled as charging.
pub fn parse_status(frame: &[u8]) -> Result<(Option<u8>, Option<bool>), &'static str> {
    if frame.len() < 14 || frame[0] != STATUS {
        return Err("Haute42 电量帧标识或长度错误");
    }
    let percent = match frame[12] {
        0xff => None,
        value @ 0..=100 => Some(value),
        _ => return Err("Haute42 电量字段超出 0–100"),
    };
    let charging = match frame[13] & 0x0f {
        0 => Some(false),
        2 => None,
        _ => Some(true),
    };
    Ok((percent, charging))
}

fn supported(item: &Interface) -> bool {
    item.vid == VID
        && item.pid == PID
        && item.usage_page == USAGE_PAGE
        && item.usage == USAGE
        && item.feature_len >= 14
        && !item
            .path
            .to_ascii_lowercase()
            .contains("{00001124-0000-1000-8000-00805f9b34fb}")
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|item| supported(item));
    let Some(interface) = matches.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(), present: false,
            error: "未找到非蓝牙 DualSense 身份下的 Haute42 Adapter 4DS 特征接口".into(),
            settings_error: "仅支持 USB 接入并通过固件串确认的 Haute42 Adapter 4DS；其他 DualSense 与蓝牙连接不会认领。".into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err("发现多个 Haute42 4DS 特征接口，无法确定唯一适配器".into());
    }
    let handle = Device::open(&interface.path)?;
    let length = interface.feature_len as usize;
    if length < 14 || length > 4096 {
        return Err("Haute42 HID Feature 报告长度异常".into());
    }
    if !parse_firmware(&handle.receive(FIRMWARE, length)?) {
        return Err("固件标识不是 Haute42；不将普通 DualSense 认作 4DS".into());
    }
    let chip = parse_chip_id(&handle.receive(DEVICE_INFO, length)?)
        .ok_or("Haute42 芯片 ID 缺失或无效，停止设备认领")?;
    let (percent, charging) = parse_status(&handle.receive(STATUS, length)?)?;
    Ok(State {
        key: DEVICE_KEY.into(), present: true, fresh: true,
        percent, charging,
        wired: true,
        updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        identity: format!("Haute42 Adapter 4DS · 芯片 {chip} · USB 054C:0CE6"),
        settings_error: "Vigil Watt 上游仅提供只读电量；此插件不提供手柄键位、宏或灯光写入。充满状态的核心字段映射为未知。".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifies_firmware_and_stable_chip_id() {
        let mut firmware = [0; 16];
        firmware[0] = FIRMWARE;
        firmware[1..9].copy_from_slice(b"Haute42 ");
        assert!(parse_firmware(&firmware));
        let mut chip = [0; 9];
        chip[0] = DEVICE_INFO;
        chip[1..].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(parse_chip_id(&chip).as_deref(), Some("0102030405060708"));
        assert!(parse_chip_id(&[DEVICE_INFO, 0, 0, 0, 0, 0, 0, 0, 0]).is_none());
    }
    #[test]
    fn parses_raw_percent_no_reading_and_charge_flags() {
        let mut frame = [0; 14];
        frame[0] = STATUS;
        frame[12] = 42;
        frame[13] = 1;
        assert_eq!(parse_status(&frame), Ok((Some(42), Some(true))));
        frame[12] = 0;
        frame[13] = 0;
        assert_eq!(parse_status(&frame), Ok((Some(0), Some(false))));
        frame[12] = 0xff;
        frame[13] = 2;
        assert_eq!(parse_status(&frame), Ok((None, None)));
        frame[12] = 101;
        assert!(parse_status(&frame).is_err());
    }
}
