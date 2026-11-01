//! Read-only report-08 battery frame from HaloBattery/providers/pulsar.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Spec {
    pub key: &'static str,
    pub name: &'static str,
    pub vid: u16,
    pub receiver: u16,
    pub wired: Option<(u16, u16)>,
}

pub fn request() -> [u8; 17] {
    let mut frame = [0; 17];
    frame[0] = 0x08;
    frame[1] = 0x04;
    frame[16] = checksum(&frame);
    frame
}
fn checksum(frame: &[u8]) -> u8 {
    frame[..16]
        .iter()
        .fold(0x55u8, |sum, byte| sum.wrapping_sub(*byte))
}
pub fn parse(frame: &[u8]) -> Option<(u8, bool)> {
    if frame.len() != 17
        || frame[0] != 0x08
        || frame[1] != 0x04
        || frame[16] != checksum(frame)
        || frame[6] > 100
    {
        return None;
    }
    Some((frame[6], frame[7] != 0))
}
fn matches(i: &hid::Interface, spec: &Spec) -> bool {
    (i.vid == spec.vid && i.pid == spec.receiver || spec.wired == Some((i.vid, i.pid)))
        && i.usage_page == 0xff02
        && i.usage == 2
        && i.output_len >= 17
        && i.input_len >= 17
}
pub fn read_state(spec: &Spec) -> Result<State, String> {
    let mut candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| matches(i, spec))
        .collect();
    candidates.sort_by_key(|i| spec.wired != Some((i.vid, i.pid)));
    let mut first_error = None;
    for i in &candidates {
        let result = (|| {
            let mut transport = WinTransport::open(&i.path)?;
            transport.flush()?;
            let mut wire = vec![0; i.output_len as usize];
            wire[..17].copy_from_slice(&request());
            transport.write(&wire, Duration::from_millis(300))?;
            let deadline = Instant::now() + Duration::from_millis(800);
            while Instant::now() < deadline {
                let reply = transport.read(i.input_len as usize, Duration::from_millis(100))?;
                if let Some(bytes) = reply.get(..17) {
                    if let Some(value) = parse(bytes) {
                        return Ok(Some(value));
                    }
                }
            }
            Ok(None)
        })();
        match result {
            Ok(Some((percent, charging))) => {
                return Ok(State {
                    key: spec.key.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    wired: spec.wired == Some((i.vid, i.pid)),
                    identity: format!("{} · {:04X}:{:04X} · FF02:0002", spec.name, i.vid, i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅支持只读电量与充电状态。".into(),
                    ..State::default()
                })
            }
            Ok(None) => {}
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
        assert_eq!(request()[16], 0x49);
        let mut r = request();
        r[6] = 75;
        r[7] = 1;
        r[16] = checksum(&r);
        assert_eq!(parse(&r), Some((75, true)));
        r[16] ^= 1;
        assert_eq!(parse(&r), None);
    }
}
