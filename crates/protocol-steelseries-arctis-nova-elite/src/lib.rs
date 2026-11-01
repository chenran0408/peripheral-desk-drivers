//! Nova Elite report facts from HaloBattery providers/steelseries_elite.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub const KEY: &str = "steelseries-arctis-nova-elite";
const LEN: usize = 64;
pub fn request() -> [u8; LEN] {
    let mut f = [0; LEN];
    f[..2].copy_from_slice(&[1, 0xb0]);
    f
}
#[derive(Default, Debug, PartialEq)]
pub struct Reading {
    pub percent: Option<u8>,
    pub charging: bool,
    pub power: Option<u8>,
}
pub fn update(reading: &mut Reading, r: &[u8]) {
    if r.len() >= 16 && r[..2] == [1, 0xb0] {
        if r[6] <= 100 {
            reading.percent = Some(r[6]);
        }
        reading.power = Some(r[14]);
        reading.charging = r[15] == 2;
    } else if r.len() >= 5 && r[..2] == [7, 0xb7] {
        if r[2] <= 100 {
            reading.percent = Some(r[2]);
            reading.charging = r[4] == 2;
        }
    } else if r.len() >= 5 && r[..2] == [7, 0xb5] {
        reading.power = Some(r[4]);
    }
}
fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x1038
        && i.pid == 0x2244
        && i.path.to_ascii_lowercase().contains("mi_03")
        && i.usage_page >= 0xff00
        && i.input_len as usize == LEN
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut choices: Vec<_> = interfaces.iter().filter(|i| candidate(i)).collect();
    choices.sort_by_key(|i| i.usage_page != 0xffc0);
    if choices.is_empty() {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 Nova Elite 基站接口 3".into(),
            ..State::default()
        });
    }
    let mut channels = Vec::new();
    for i in choices {
        channels.push(WinTransport::open(&i.path)?);
    }
    for channel in &mut channels {
        channel.flush()?;
    }
    channels[0].write(&request(), Duration::from_millis(500))?;
    let mut reading = Reading::default();
    let end = Instant::now() + Duration::from_secs(1);
    while Instant::now() < end {
        for channel in &mut channels {
            update(&mut reading, &channel.read(LEN, Duration::from_millis(50))?);
            if reading.percent.is_some() && reading.power.is_some() {
                break;
            }
        }
        if reading.percent.is_some() && reading.power.is_some() {
            break;
        }
    }
    if reading.power == Some(1) {
        return Err("Nova Elite 耳机已关机或离线".into());
    }
    let percent = reading.percent.ok_or("Nova Elite 基站未回复有效电量")?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: Some(reading.charging || reading.power == Some(2)),
        wired: false,
        identity: "SteelSeries Arctis Nova Elite · 1038:2244 · 接口 3".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅提供耳机只读电量；不显示基站备用电池。".into(),
        ..State::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_shapes() {
        let mut x = Reading::default();
        let mut direct = [0; 64];
        direct[..2].copy_from_slice(&[1, 0xb0]);
        direct[6] = 31;
        direct[14] = 8;
        update(&mut x, &direct);
        assert_eq!(x.percent, Some(31));
        update(&mut x, &[7, 0xb7, 42, 90, 2]);
        assert_eq!(x.percent, Some(42));
        assert!(x.charging);
        update(&mut x, &[7, 0xb5, 0, 0, 1]);
        assert_eq!(x.power, Some(1));
    }
}
