//! Aerox 9 battery protocol facts from rivalcfg (WTFPL) and HaloBattery (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub const KEY: &str = "steelseries-aerox-9-wireless";
const LEN: usize = 64;

pub fn command(pid: u16) -> u8 {
    if matches!(pid, 0x1858 | 0x1874) {
        0x92 | 0x40
    } else {
        0x92
    }
}
pub fn request(pid: u16) -> [u8; LEN] {
    let mut f = [0; LEN];
    f[1] = command(pid);
    f
}
pub fn parse(r: &[u8], cmd: u8) -> Option<(u8, bool)> {
    let data = if r.first() == Some(&0) {
        r.get(1..)
    } else {
        Some(r)
    }?;
    if data.len() < 2 || data[0] != cmd {
        return None;
    }
    let level = data[1] & 0x7f;
    if !(1..=21).contains(&level) {
        return None;
    }
    Some(((level - 1) * 5, data[1] & 0x80 != 0))
}
fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x1038
        && matches!(i.pid, 0x1858 | 0x1874 | 0x185a | 0x1876)
        && i.usage_page == 0xffc0
        && i.input_len as usize >= LEN
        && i.output_len as usize >= LEN
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut choices: Vec<_> = interfaces.iter().filter(|i| candidate(i)).collect();
    choices.sort_by_key(|i| matches!(i.pid, 0x1858 | 0x1874));
    if choices.is_empty() {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 Aerox 9 Wireless 指定电量接口".into(),
            ..State::default()
        });
    }
    for i in choices {
        let mut transport = WinTransport::open(&i.path)?;
        transport.flush()?;
        let cmd = command(i.pid);
        transport.write(&request(i.pid), Duration::from_millis(500))?;
        let end = Instant::now() + Duration::from_millis(1000);
        while Instant::now() < end {
            // Receiver reports can be 64 bytes; only the command and level are decoded.
            if let Some((percent, charging)) = parse(
                &transport.read(i.input_len as usize, Duration::from_millis(100))?,
                cmd,
            ) {
                return Ok(State {
                    key: KEY.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    wired: matches!(i.pid, 0x185a | 0x1876),
                    identity: format!("SteelSeries Aerox 9 Wireless · 1038:{:04X}", i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅提供只读电量与充电状态。".into(),
                    ..State::default()
                });
            }
        }
    }
    Err("Aerox 9 Wireless 未收到匹配的电量回复".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn steps_and_mode() {
        assert_eq!(command(0x1858), 0xd2);
        assert_eq!(command(0x185a), 0x92);
        assert_eq!(parse(&[0xd2, 0x95], 0xd2), Some((100, true)));
        assert_eq!(parse(&[0, 0x92, 1], 0x92), Some((0, false)));
        assert_eq!(parse(&[0xd2, 0], 0xd2), None);
    }
}
