//! Ninjutso Sora V2 read-only feature report from lllexalllepexa/sora tray.
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VID: u16 = 0x1915;
const DONGLE_PID: u16 = 0xae1c;
const WIRED_PID: u16 = 0xae12;
const USAGE_PAGE: u16 = 0xffa0;
const REPORT_ID: u8 = 0x05;
const REPORT_LEN: usize = 32;

pub fn request() -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[0] = REPORT_ID;
    frame[1] = 0x15;
    frame[4] = 0x01;
    frame[7] = 0x04;
    frame
}

pub fn parse(reply: &[u8]) -> Option<(u8, bool, bool, bool)> {
    if reply.len() != REPORT_LEN || reply[0] != REPORT_ID || reply[1] != 0x15 {
        return None;
    }
    let percent = reply[9];
    if !(1..=100).contains(&percent) || reply[10] > 1 || reply[11] > 1 || reply[12] > 1 {
        return None;
    }
    Some((percent, reply[10] == 1, reply[11] == 1, reply[12] == 1))
}

pub fn read_state() -> Result<State, String> {
    let mut interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|interface| {
            interface.vid == VID
                && matches!(interface.pid, DONGLE_PID | WIRED_PID)
                && interface.usage_page == USAGE_PAGE
                && interface.feature_len as usize >= REPORT_LEN
        })
        .collect();
    interfaces.sort_by_key(|interface| interface.pid != WIRED_PID);
    if interfaces.is_empty() {
        return Ok(State {
            key: "ninjutso-sora-v2".into(),
            error: "未找到 Ninjutso Sora V2 Feature 05 接口".into(),
            ..State::default()
        });
    }
    for interface in interfaces {
        let device = hid::Device::open(&interface.path)?;
        device.send(&request())?;
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(30));
            let reply = device.receive(REPORT_ID, REPORT_LEN)?;
            let Some((percent, charging, full, online)) = parse(&reply) else {
                continue;
            };
            if interface.pid == DONGLE_PID && !online {
                return Ok(State {
                    key: "ninjutso-sora-v2".into(),
                    present: true,
                    fresh: true,
                    identity: "Ninjutso Sora V2 · 1915:AE1C · 鼠标未连接".into(),
                    settings_error: "接收器未报告已连接鼠标；未显示电量。".into(),
                    updated: now(),
                    ..State::default()
                });
            }
            return Ok(State {
                key: "ninjutso-sora-v2".into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: interface.pid == WIRED_PID,
                identity: format!("Ninjutso Sora V2 · 1915:{:04X}", interface.pid),
                settings_error: if charging || full {
                    "充电时上游的百分比可能仍是插线前读数；仅反映当前上游回复。".into()
                } else {
                    "仅提供只读电量和上游充电标志。".into()
                },
                updated: now(),
                ..State::default()
            });
        }
    }
    Err("Ninjutso Sora V2 未返回有效的 Feature 05 电量数据".into())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_validates_feature_reply() {
        assert_eq!(request()[..8], [5, 0x15, 0, 0, 1, 0, 0, 4]);
        let mut reply = [0; REPORT_LEN];
        reply[0] = REPORT_ID;
        reply[1] = 0x15;
        reply[9] = 88;
        reply[10] = 1;
        reply[11] = 0;
        reply[12] = 1;
        assert_eq!(parse(&reply), Some((88, true, false, true)));
        reply[9] = 101;
        assert_eq!(parse(&reply), None);
        assert!(parse(&reply[..31]).is_none());
    }
}
