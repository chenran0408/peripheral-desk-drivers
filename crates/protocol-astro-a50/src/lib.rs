//! Astro A50 Gen 5 exchange from HaloBattery providers/astro.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "astro-a50-gen5";
const LEN: usize = 64;

pub fn request() -> [u8; LEN] {
    let mut data = [0; LEN];
    data[..6].copy_from_slice(&[2, 0x0c, 3, 0, 6, 0x0c]);
    data
}

pub fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    (reply.len() > 8 && reply[0] == 2 && reply[1] == 0x0c && reply[4] == 6 && reply[6] <= 100)
        .then(|| (reply[6], reply[8] != 0))
}

pub fn read_state() -> Result<State, String> {
    let interface = hid::enumerate()?
        .into_iter()
        .find(|i| i.vid == 0x046d && i.pid == 0x0b1c && i.usage_page == 0xff32 && i.usage == 0x74);
    let Some(i) = interface else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 A50 Gen 5 基座控制接口".into(),
            ..State::default()
        });
    };
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    io.write(&request(), Duration::from_millis(500))?;
    for _ in 0..8 {
        if let Some((percent, charging)) = parse(&io.read(LEN, Duration::from_millis(300))?) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                identity: "Astro A50 Gen 5 · 046D:0B1C".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅支持只读电量与充电状态。".into(),
                ..State::default()
            });
        }
    }
    Err("A50 基座未返回有效电量".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn battery() {
        assert_eq!(&request()[..6], &[2, 12, 3, 0, 6, 12]);
        let mut reply = [2, 12, 6, 0, 6, 12, 80, 80, 1];
        assert_eq!(parse(&reply), Some((80, true)));
        reply[6] = 101;
        assert_eq!(parse(&reply), None);
    }
}
