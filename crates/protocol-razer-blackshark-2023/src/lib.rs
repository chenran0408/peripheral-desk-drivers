//! Razer BlackShark V2 Pro (2023) exact USB receiver PA battery protocol.
//! Adapted from Vigil Watt v1.3.0 `RazerHeadsetCodec.cs` and its scoped session
//! in `RazerPlugin.cs`, commit a78e973b4eb634bdce37c9078dd48d1c07e3eb73.
//! Copyright (c) 2026 evren; MIT license retained with the driver package.

use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "razer-blackshark-v2-pro-2023";
pub const VID: u16 = 0x1532;
pub const PID: u16 = 0x0555;
pub const USAGE_PAGE: u16 = 0xff00;
pub const USAGE: u16 = 1;
pub const REPORT_LEN: usize = 64;
const RID: u8 = 0x02;
const BATTERY: u8 = 0x21;
const CHARGING: u8 = 0x2a;

pub fn query(command: u8) -> Option<[u8; REPORT_LEN]> {
    if !matches!(command, BATTERY | CHARGING) {
        return None;
    }
    let mut frame = [0; REPORT_LEN];
    frame[0] = RID;
    frame[1] = 0x80;
    frame[2] = 8;
    frame[5] = 0x50;
    frame[6] = 0x41;
    frame[7] = 8;
    frame[9] = 3;
    frame[10] = command;
    Some(frame)
}

pub fn remote_mode(enabled: bool) -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[0] = RID;
    frame[1] = 0x80;
    frame[2] = 7;
    frame[5] = 0x50;
    frame[6] = 0x41;
    frame[7] = 0x0e;
    frame[9] = 2;
    frame[10] = 0xe1;
    frame[11] = u8::from(enabled);
    frame
}

fn parse_reply(frame: &[u8], command: u8) -> Option<u8> {
    (frame.len() == REPORT_LEN
        && frame[0] == RID
        && frame[2] == 0x50
        && frame[3] == 0x49
        && frame[12] == command
        && frame[13] == 1
        && frame[14] == 1)
        .then_some(frame[15])
}

pub fn parse_battery(frame: &[u8]) -> Option<u8> {
    let value = parse_reply(frame, BATTERY)?;
    (value <= 100).then_some(value)
}

pub fn parse_charging(frame: &[u8]) -> Option<bool> {
    parse_reply(frame, CHARGING).map(|value| value != 0)
}

fn supported(item: &Interface) -> bool {
    item.vid == VID
        && item.pid == PID
        && item.usage_page == USAGE_PAGE
        && item.usage == USAGE
        && item.input_len as usize == REPORT_LEN
        && item.output_len as usize == REPORT_LEN
        && item.path.to_ascii_lowercase().contains("&mi_03&col02#")
}

fn query_one(transport: &mut WinTransport, command: u8) -> Result<u8, String> {
    for _ in 0..8 {
        if transport
            .read(REPORT_LEN, Duration::from_millis(1))?
            .is_empty()
        {
            break;
        }
    }
    let mut remote_authorized = false;
    let result = (|| {
        for _ in 0..2 {
            transport.write(&remote_mode(true), Duration::from_millis(500))?;
            remote_authorized = true;
            std::thread::sleep(Duration::from_millis(35));
        }
        let request = query(command).ok_or("不允许的 Razer PA 查询命令")?;
        transport.write(&request, Duration::from_millis(500))?;
        let deadline = Instant::now() + Duration::from_millis(700);
        while Instant::now() < deadline {
            let report = transport.read(REPORT_LEN, Duration::from_millis(100))?;
            if let Some(value) = parse_reply(&report, command) {
                if command == BATTERY && value > 100 {
                    return Err("Razer 电量超出 0–100".into());
                }
                return Ok(value);
            }
        }
        Err("Razer BlackShark 2.4 GHz 查询无有效应答".into())
    })();
    if remote_authorized {
        let release = transport.write(&remote_mode(false), Duration::from_millis(500));
        if release.is_err() {
            return Err("Razer 远程会话释放失败".into());
        }
    }
    result
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|item| supported(item));
    let Some(interface) = matches.next() else {
        return Ok(State { key: DEVICE_KEY.into(), present: false,
            error: "未找到 Razer BlackShark V2 Pro (2023) 2.4 GHz PA 接口".into(),
            settings_error: "仅支持 1532:0555 接收器 MI_03&Col02；不适用于 2020 版 0528 或 0556。查询写帧需在驱动管理中明确启用。".into(), ..State::default() });
    };
    if matches.next().is_some() {
        return Err("匹配到多个 BlackShark PA 接口，停止查询".into());
    }
    let mut transport = WinTransport::open(&interface.path)?;
    let percent = query_one(&mut transport, BATTERY)?;
    let charging = query_one(&mut transport, CHARGING)? != 0;
    Ok(State { key: DEVICE_KEY.into(), present: true, fresh: true,
        percent: Some(percent), charging: Some(charging), wired: false,
        updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        identity: "Razer BlackShark V2 Pro (2023) · 2.4 GHz · 1532:0555 · PA".into(),
        settings_error: "上游只移植电量与充电查询；EQ、音量、麦克风及其他设备写入未开放。每次刷新会建立两次短时查询会话。".into(), ..State::default() })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn reply(command: u8, value: u8) -> [u8; REPORT_LEN] {
        let mut frame = [0; REPORT_LEN];
        frame[0] = RID;
        frame[2] = 0x50;
        frame[3] = 0x49;
        frame[12] = command;
        frame[13] = 1;
        frame[14] = 1;
        frame[15] = value;
        frame
    }
    #[test]
    fn frames_match_upstream_pa_commands() {
        let frame = query(BATTERY).unwrap();
        assert_eq!(
            &frame[..12],
            &[2, 0x80, 8, 0, 0, 0x50, 0x41, 8, 0, 3, 0x21, 0]
        );
        assert_eq!(
            remote_mode(true)[..12],
            [2, 0x80, 7, 0, 0, 0x50, 0x41, 0x0e, 0, 2, 0xe1, 1]
        );
    }
    #[test]
    fn reads_only_exact_replies_and_preserves_charging_unknown() {
        assert_eq!(parse_battery(&reply(BATTERY, 79)), Some(79));
        assert_eq!(parse_battery(&reply(BATTERY, 101)), None);
        assert_eq!(parse_charging(&reply(CHARGING, 0)), Some(false));
        assert_eq!(parse_charging(&reply(CHARGING, 1)), Some(true));
        assert_eq!(parse_charging(&reply(CHARGING, 2)), Some(true));
        assert_eq!(parse_battery(&reply(CHARGING, 80)), None);
    }
}
