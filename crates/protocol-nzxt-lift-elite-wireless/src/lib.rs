//! Protocol facts from HaloBattery providers/nzxt.py (MIT), independently implemented.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "nzxt-lift-elite-wireless";
const LEN: usize = 64;

pub fn request() -> [u8; LEN] {
    let mut frame = [0; LEN];
    frame[..5].copy_from_slice(&[0x4e, 2, 0x81, 0, 0xb0]);
    frame
}

pub fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    if reply.len() < 27 || reply[..3] != [0x4e, 2, 0x97] {
        return None;
    }
    let percent = u16::from_le_bytes([reply[17], reply[18]]);
    (percent <= 100).then_some((percent as u8, reply[22] & 1 != 0))
}

fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x1e71
        && matches!(i.pid, 0x2101 | 0x2129)
        && i.usage_page == 0xffca
        && i.usage == 1
        && i.input_len as usize == LEN
        && i.output_len as usize == LEN
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut choices: Vec<_> = interfaces.iter().filter(|i| candidate(i)).collect();
    choices.sort_by_key(|i| i.pid != 0x2129);
    if choices.is_empty() {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 NZXT Lift Elite 指定电量接口".into(),
            ..State::default()
        });
    }
    let mut reading = None;
    for i in choices {
        let mut transport = WinTransport::open(&i.path)?;
        transport.flush()?;
        transport.write(&request(), Duration::from_millis(500))?;
        let end = Instant::now() + Duration::from_millis(1500);
        while Instant::now() < end {
            let reply = transport.read(LEN, Duration::from_millis(150))?;
            if reply.starts_with(&[0x4e, 0xe5]) {
                continue;
            }
            if let Some(value) = parse(&reply) {
                reading = Some((value, i.pid));
                break;
            }
        }
        if reading.is_some() {
            break;
        }
    }
    let ((percent, charging), pid) =
        reading.ok_or("NZXT 电量查询无有效回复；NZXT CAM 同时运行时接收器可能不回应")?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: Some(charging),
        wired: pid == 0x2129,
        identity: format!("NZXT Lift Elite Wireless · 1E71:{pid:04X} · FFCA:0001"),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅支持只读电量与接线充电状态。".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn telemetry() {
        let mut frame = [0; 64];
        frame[..3].copy_from_slice(&[0x4e, 2, 0x97]);
        frame[17] = 77;
        frame[22] = 1;
        assert_eq!(parse(&frame), Some((77, true)));
        frame[18] = 1;
        assert_eq!(parse(&frame), None);
        assert_eq!(parse(&[0x4e, 0xe5]), None);
        assert_eq!(&request()[..5], &[0x4e, 2, 0x81, 0, 0xb0]);
    }
}
