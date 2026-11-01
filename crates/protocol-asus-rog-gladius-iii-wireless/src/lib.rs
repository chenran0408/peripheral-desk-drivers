//! Protocol facts from rogdrv-web (GPL-3.0); no upstream source code copied.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub const KEY: &str = "asus-rog-gladius-iii-wireless";
const LEN: usize = 64;

pub fn request() -> [u8; LEN] {
    let mut f = [0; LEN];
    f[0] = 0x12;
    f[1] = 0x07;
    f
}
pub fn parse(r: &[u8]) -> Option<(u8, bool)> {
    (r.len() >= 10 && r[0] == 0x12 && r[1] == 0x07 && r[4] <= 100).then_some((
        r.get(4).copied().unwrap_or(0),
        r.get(9).copied().unwrap_or(0) != 0,
    ))
}
fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x0b05
        && matches!(i.pid, 0x1a72 | 0x1a70)
        && i.usage_page == 0xff01
        && i.usage == 1
        && i.input_len as usize == LEN
        && i.output_len as usize == LEN
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut choices: Vec<_> = interfaces.iter().filter(|i| candidate(i)).collect();
    choices.sort_by_key(|i| i.pid != 0x1a70);
    if choices.is_empty() {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 ROG Gladius III Wireless 指定 FF01 电量接口".into(),
            ..State::default()
        });
    }
    for i in choices {
        let mut transport = WinTransport::open(&i.path)?;
        transport.flush()?;
        transport.write(&request(), Duration::from_millis(500))?;
        let end = Instant::now() + Duration::from_millis(1000);
        while Instant::now() < end {
            if let Some((percent, charging)) =
                parse(&transport.read(LEN, Duration::from_millis(100))?)
            {
                return Ok(State {
                    key: KEY.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    wired: i.pid == 0x1a70,
                    identity: format!("ROG Gladius III Wireless AimPoint 36K · 0B05:{:04X}", i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅提供只读电量；蓝牙连接不支持。".into(),
                    ..State::default()
                });
            }
        }
    }
    Err("ROG Gladius III Wireless 未收到匹配的电量回复".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response() {
        let mut f = [0; LEN];
        f[0] = 0x12;
        f[1] = 7;
        f[4] = 65;
        f[9] = 1;
        assert_eq!(parse(&f), Some((65, true)));
        assert_eq!(&request()[..2], &[0x12, 7]);
    }
}
