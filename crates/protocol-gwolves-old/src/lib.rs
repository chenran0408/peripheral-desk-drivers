//! Old G-Wolves feature exchange from HaloBattery/providers/gwolves.py and mouse.xyz.
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "gwolves-hsk-pro-ace";
pub fn request(len: usize, wired: bool) -> Vec<u8> {
    let mut frame = vec![0; len];
    frame[..5].copy_from_slice(&[0, 0, 2, 0x8f, u8::from(!wired)]);
    frame
}
pub fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    let at = if reply.first() == Some(&0xa1) {
        0
    } else if reply.get(1) == Some(&0xa1) {
        1
    } else {
        return None;
    };
    if reply.get(at + 1..at + 3) != Some(&[2, 0x8f]) {
        return None;
    }
    let percent = *reply.get(at + 5)?;
    (percent <= 100).then_some((percent, *reply.get(at + 4)? != 0))
}
pub fn read_state() -> Result<State, String> {
    let mut candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x33e4
                && matches!(i.pid, 0x5803 | 0x5804)
                && i.feature_len == 65
                && i.usage_page >= 0xff00
        })
        .collect();
    candidates.sort_by_key(|i| i.pid != 0x5804);
    let mut first_error = None;
    for i in &candidates {
        let result = (|| {
            let device = hid::Device::open(&i.path)?;
            device.send(&request(i.feature_len as usize, i.pid == 0x5804))?;
            for _ in 0..15 {
                std::thread::sleep(Duration::from_millis(50));
                if let Ok(reply) = device.receive(0, i.feature_len as usize) {
                    if parse(&reply).is_some() {
                        return Ok(reply);
                    }
                }
            }
            Ok(Vec::new())
        })();
        match result {
            Ok(reply) => {
                if let Some((percent, charging)) = parse(&reply) {
                    return Ok(State {
                        key: KEY.into(),
                        present: true,
                        fresh: true,
                        percent: Some(percent),
                        charging: Some(charging),
                        identity: format!("G-Wolves HSK Pro ACE · 33E4:{:04X}", i.pid),
                        wired: i.pid == 0x5804,
                        updated: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        settings_error: "仅支持只读电量与充电状态。".into(),
                        ..State::default()
                    });
                }
            }
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }
    Ok(State {
        key: KEY.into(),
        present: !candidates.is_empty(),
        error: first_error.unwrap_or_else(|| "HSK Pro ACE 未取得有效电量回复".into()),
        ..State::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame() {
        assert_eq!(&request(65, false)[..5], &[0, 0, 2, 0x8f, 1]);
        assert_eq!(parse(&[0xa1, 2, 0x8f, 0, 1, 70]), Some((70, true)));
    }
}
