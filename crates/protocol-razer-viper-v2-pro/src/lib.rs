//! Read-only Razer battery queries for one explicitly matched USB interface.
//!
//! Protocol facts are derived from OpenRazer at commit
//! `477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`; see docs/RAZER-VIPER-V2-PRO.md.
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::State,
};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const DEVICE_KEY: &str = "razer-viper-v2-pro-wired";
const VID: u16 = 0x1532;
const PID: u16 = 0x00a5;
const REPORT_LEN: usize = 91;
const TRANSACTION_ID: u8 = 0x1f;
const COMMAND_CLASS: u8 = 0x07;
const BATTERY_COMMAND: u8 = 0x80;
const CHARGING_COMMAND: u8 = 0x84;

fn matching_interface(interface: &Interface) -> bool {
    interface.vid == VID
        && interface.pid == PID
        && interface.feature_len == REPORT_LEN as u16
        && interface.path.to_ascii_lowercase().contains("mi_00")
}

fn request(command: u8) -> [u8; REPORT_LEN] {
    let mut wire = [0; REPORT_LEN];
    wire[2] = TRANSACTION_ID;
    wire[6] = 2;
    wire[7] = COMMAND_CLASS;
    wire[8] = command;
    wire[89] = checksum(&wire);
    wire
}

fn checksum(wire: &[u8; REPORT_LEN]) -> u8 {
    wire[3..89].iter().fold(0, |sum, byte| sum ^ byte)
}

fn reply_value(wire: &[u8], request: &[u8; REPORT_LEN]) -> Result<u8, String> {
    let Ok(wire) = <&[u8; REPORT_LEN]>::try_from(wire) else {
        return Err("Razer Feature 回复长度不匹配".into());
    };
    if wire[0] != 0 || wire[89] != checksum(wire) {
        return Err("Razer Feature 回复的 Report ID 或校验不匹配".into());
    }
    if wire[2] != request[2]
        || wire[3..5] != request[3..5]
        || wire[5] != 0
        || wire[6] != request[6]
        || wire[7] != request[7]
        || wire[8] != request[8]
    {
        return Err("Razer Feature 回复不属于本次只读查询".into());
    }
    match wire[1] {
        0x02 => Ok(wire[10]),
        0x01 => Err("Razer 设备仍在处理查询".into()),
        status => Err(format!("Razer 设备未完成查询（状态 0x{status:02X}）")),
    }
}

fn query(device: &Device, command: u8) -> Result<u8, String> {
    let request = request(command);
    let mut last_error = String::new();
    for attempt in 0..5 {
        device.send(&request)?;
        thread::sleep(Duration::from_millis(31));
        let response = device.receive(0, REPORT_LEN)?;
        match reply_value(&response, &request) {
            Ok(value) => return Ok(value),
            Err(error) => last_error = error,
        }
        if attempt < 4 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    Err(last_error)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_charging(raw: u8) -> Result<bool, String> {
    match raw {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(format!("Razer 返回未知充电状态 0x{value:02X}")),
    }
}

fn base_state(present: bool) -> State {
    State {
        key: DEVICE_KEY.into(),
        present,
        wired: true,
        identity: "Razer Viper V2 Pro wired · USB 1532:00A5 · TID 0x1F".into(),
        settings_error: "仅开放只读电量和充电状态；DPI、按键、宏和灯光未开放。".into(),
        updated: now(),
        ..Default::default()
    }
}

/// Reads battery and charging state only from the unique known feature collection.
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let matches: Vec<_> = interfaces
        .iter()
        .filter(|i| matching_interface(i))
        .collect();
    match matches.as_slice() {
        [] => {
            let mut state = base_state(false);
            state.error =
                "未找到唯一的 Viper V2 Pro 有线 Feature 接口（1532:00A5 / MI_00 / 91 字节）".into();
            Ok(state)
        }
        [interface] => {
            let device = Device::open(&interface.path)?;
            let battery = match query(&device, BATTERY_COMMAND) {
                Ok(raw) => raw,
                Err(error) => {
                    let mut state = base_state(true);
                    state.error = error;
                    return Ok(state);
                }
            };
            let charging = query(&device, CHARGING_COMMAND).and_then(parse_charging);
            let mut state = base_state(true);
            state.fresh = true;
            state.percent = Some(((u16::from(battery) * 100 + 127) / 255) as u8);
            match charging {
                Ok(value) => state.charging = Some(value),
                Err(error) => state.error = format!("电量已读取；充电状态不可用：{error}"),
            }
            Ok(state)
        }
        _ => Err("找到多个 Viper V2 Pro Feature 接口；为避免向错误接口发送请求，已停止读取".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn success_reply(request: &[u8; REPORT_LEN], value: u8) -> [u8; REPORT_LEN] {
        let mut wire = *request;
        wire[1] = 2;
        wire[8] |= 0x80;
        wire[10] = value;
        wire[89] = checksum(&wire);
        wire
    }

    #[test]
    fn read_commands_are_exact_and_read_only() {
        for command in [BATTERY_COMMAND, CHARGING_COMMAND] {
            let wire = request(command);
            assert_eq!(&wire[..9], &[0, 0, 0x1f, 0, 0, 0, 2, 7, command]);
            assert_eq!(wire[89], checksum(&wire));
            assert_eq!(wire[90], 0);
        }
    }

    #[test]
    fn reply_must_match_transaction_and_checksum() {
        let request = request(BATTERY_COMMAND);
        let mut reply = success_reply(&request, 128);
        assert_eq!(reply_value(&reply, &request).unwrap(), 128);
        reply[2] ^= 1;
        reply[89] = checksum(&reply);
        assert!(reply_value(&reply, &request).is_err());
        reply[2] = request[2];
        reply[10] ^= 1;
        assert!(reply_value(&reply, &request).is_err());
    }

    #[test]
    fn charging_state_rejects_unknown_values() {
        assert!(!parse_charging(0).unwrap());
        assert!(parse_charging(1).unwrap());
        assert!(parse_charging(2).is_err());
    }

    #[test]
    fn only_exact_wired_hid_collection_is_accepted() {
        let mut interface = Interface {
            path: r"\\?\hid#vid_1532&pid_00a5&mi_00".into(),
            vid: VID,
            pid: PID,
            usage_page: 0xff00,
            usage: 1,
            feature_len: REPORT_LEN as u16,
            input_len: 0,
            output_len: 0,
            product: "Razer Viper V2 Pro".into(),
        };
        assert!(matching_interface(&interface));
        interface.pid = 0x00a6;
        assert!(!matching_interface(&interface));
        interface.pid = PID;
        interface.path = r"\\?\hid#vid_1532&pid_00a5&mi_01".into();
        assert!(!matching_interface(&interface));
        interface.path = r"\\?\hid#vid_1532&pid_00a5&mi_00".into();
        interface.feature_len = 90;
        assert!(!matching_interface(&interface));
    }

    #[test]
    fn reply_with_a_write_command_is_rejected_even_with_a_valid_checksum() {
        let request = request(BATTERY_COMMAND);
        let mut reply = success_reply(&request, 80);
        reply[8] = 0x00;
        reply[89] = checksum(&reply);
        assert!(reply_value(&reply, &request).is_err());
    }
}
