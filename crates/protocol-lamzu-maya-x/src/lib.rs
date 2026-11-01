//! Protocol facts from HaloBattery providers/lamzu.py (MIT), independently implemented.
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "lamzu-maya-x";
const LEN: usize = 64;

pub fn request() -> [u8; LEN] {
    let mut frame = [0; LEN];
    frame[..6].copy_from_slice(&[0, 0, 2, 2, 0, 0x83]);
    frame
}
pub fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    (reply.len() >= 8 && reply[..6] == [0xa1, 0, 2, 2, 0, 0x83] && reply[7] <= 100).then_some((
        reply.get(7).copied().unwrap_or(0),
        reply.get(6).copied().unwrap_or(0) != 0,
    ))
}
fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x373e
        && matches!(i.pid, 0x001e | 0x001c)
        && i.usage_page == 0xffff
        && i.usage == 0
        && i.path.to_ascii_lowercase().contains("mi_02")
        && i.feature_len as usize >= LEN
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut choices: Vec<_> = interfaces.iter().filter(|i| candidate(i)).collect();
    choices.sort_by_key(|i| i.pid != 0x001c);
    if choices.is_empty() {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 LAMZU Maya X 接口 2 Feature 通道".into(),
            ..State::default()
        });
    }
    for i in choices {
        let device = hid::Device::open(&i.path)?;
        device.send(&request())?;
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(50));
            if let Some((percent, charging)) = parse(&device.receive(0, LEN)?) {
                return Ok(State {
                    key: KEY.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    wired: i.pid == 0x001c,
                    identity: format!("LAMZU Maya X · 373E:{:04X} · FFFF:0000", i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅提供只读电量；有线 ID 上游未单独确认。".into(),
                    ..State::default()
                });
            }
        }
    }
    Err("LAMZU Maya X 未收到 0x83 电量 Feature 回复".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feature_reply() {
        assert_eq!(&request()[..6], &[0, 0, 2, 2, 0, 0x83]);
        assert_eq!(parse(&[0xa1, 0, 2, 2, 0, 0x83, 1, 58]), Some((58, true)));
        assert_eq!(parse(&[0xa1, 0, 2, 2, 0, 0x83, 0, 101]), None);
    }
}
