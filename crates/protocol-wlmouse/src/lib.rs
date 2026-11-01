//! Read-only WLmouse feature exchange from HaloBattery/providers/wlmouse.py (MIT).
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Spec {
    pub key: &'static str,
    pub name: &'static str,
    pub pid: u16,
}
pub fn request(len: usize) -> Vec<u8> {
    let mut frame = vec![0; len];
    frame[..6].copy_from_slice(&[0, 0, 2, 2, 0, 0x83]);
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
    if reply.get(at + 1..at + 6) != Some(&[0, 2, 2, 0, 0x83]) {
        return None;
    }
    let percent = *reply.get(at + 7)?;
    (percent <= 100).then_some((percent, *reply.get(at + 6)? != 0))
}
pub fn read_state(spec: &Spec) -> Result<State, String> {
    let candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x36a7
                && i.pid == spec.pid
                && i.usage_page == 0xffff
                && i.usage == 0
                && i.feature_len == 65
        })
        .collect();
    let mut first_error = None;
    for i in &candidates {
        let result = (|| {
            let device = hid::Device::open(&i.path)?;
            device.send(&request(i.feature_len as usize))?;
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
                        key: spec.key.into(),
                        present: true,
                        fresh: true,
                        percent: Some(percent),
                        charging: Some(charging),
                        identity: format!("{} · 36A7:{:04X}", spec.name, i.pid),
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
        key: spec.key.into(),
        present: !candidates.is_empty(),
        error: first_error.unwrap_or_else(|| format!("{} 未取得有效电量回复", spec.name)),
        ..State::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame() {
        assert_eq!(&request(64)[..6], &[0, 0, 2, 2, 0, 0x83]);
        assert_eq!(parse(&[0xa1, 0, 2, 2, 0, 0x83, 1, 87]), Some((87, true)));
    }
}
