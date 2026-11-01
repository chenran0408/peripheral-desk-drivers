//! Dark Core battery route facts from HaloBattery providers/corsair.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub const KEY: &str = "corsair-dark-core-rgb-pro-se";
const WIRE_LEN: usize = 65;

pub fn request() -> [u8; WIRE_LEN] {
    let mut f = [0; WIRE_LEN];
    f[1..4].copy_from_slice(&[9, 2, 0x0f]);
    f
}
pub fn parse(r: &[u8]) -> Option<u8> {
    let data = if r.len() >= WIRE_LEN && r[0] == 0 {
        &r[1..]
    } else {
        r
    };
    if data.len() < 5 || data[..3] != [1, 2, 0] {
        return None;
    }
    let tenth = u16::from_le_bytes([data[3], data[4]]);
    (1..=1000).contains(&tenth).then_some((tenth / 10) as u8)
}
fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x1b1c && i.pid == 0x1b7f && i.usage_page == 0xff42 && i.usage == 1
}
fn one(transport: &mut WinTransport, window: Duration) -> Result<Option<u8>, String> {
    transport.write(&request(), Duration::from_millis(500))?;
    let end = Instant::now() + window;
    while Instant::now() < end {
        if let Some(percent) = parse(&transport.read(65, Duration::from_millis(150))?) {
            return Ok(Some(percent));
        }
    }
    Ok(None)
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let i = match interfaces.iter().find(|i| candidate(i)) {
        Some(i) => i,
        None => {
            return Ok(State {
                key: KEY.into(),
                error: "未找到 Corsair Dark Core RGB Pro SE 指定接收器".into(),
                ..State::default()
            })
        }
    };
    let mut transport = WinTransport::open(&i.path)?;
    transport.flush()?;
    let first =
        one(&mut transport, Duration::from_secs(6))?.ok_or("Corsair 接收器未回复电量属性")?;
    let second =
        one(&mut transport, Duration::from_secs(3))?.ok_or("Corsair 电量回复无法连续确认")?;
    if first != second {
        return Err("Corsair 两次电量回复不一致，可能混入其他属性回复".into());
    }
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(first),
        charging: None,
        wired: false,
        identity: "Corsair Dark Core RGB Pro SE · 1B1C:1B7F · FF42:0001".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅提供只读电量；此回复不报告充电状态。".into(),
        ..State::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routed_reply() {
        assert_eq!(&request()[..4], &[0, 9, 2, 15]);
        assert_eq!(parse(&[1, 2, 0, 0x26, 0x02]), Some(55));
        assert_eq!(parse(&[1, 2, 0, 0, 0]), None);
    }
}
