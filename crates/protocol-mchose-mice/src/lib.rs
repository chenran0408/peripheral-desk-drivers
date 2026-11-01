//! Read-only MCHOSE mouse protocols from HaloBattery/providers/mchose.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub enum Model {
    A5,
    A7,
    G7,
    M7,
}

impl Model {
    fn spec(self) -> (&'static str, &'static str, u16, &'static [u16]) {
        match self {
            Self::A5 => (
                "mchose-a5-pro-max",
                "MCHOSE A5 Pro Max",
                0x2023,
                &[0xf013, 0xf015, 0xf019],
            ),
            Self::A7 => (
                "mchose-a7-v2-ultra",
                "MCHOSE A7 V2 Ultra",
                0x3837,
                &[0x100b],
            ),
            Self::G7 => ("mchose-g7", "MCHOSE G7", 0xa8a5, &[0x2255]),
            Self::M7 => (
                "mchose-m7-ultra",
                "MCHOSE M7 Ultra",
                0x5253,
                &[0x1020, 0x0031],
            ),
        }
    }
    fn matches(self, i: &hid::Interface) -> bool {
        let (_, _, vid, pids) = self.spec();
        if i.vid != vid || !pids.contains(&i.pid) {
            return false;
        }
        match self {
            Self::A5 => i.usage_page == 0xffff && i.feature_len >= 64,
            Self::G7 => i.usage_page == 0xff01 && i.output_len >= 65 && i.input_len >= 12,
            Self::A7 | Self::M7 => i.usage_page == 0xff01 && i.usage == 1 && i.feature_len >= 21,
        }
    }
}

pub fn a5_request() -> [u8; 65] {
    let mut out = [0; 65];
    out[..7].copy_from_slice(&[0, 0, 0, 2, 2, 0, 0x83]);
    out
}
pub fn parse_a5(r: &[u8]) -> Option<(u8, bool)> {
    let at = if r.first() == Some(&0xa1) {
        0
    } else if r.get(1) == Some(&0xa1) {
        1
    } else {
        return None;
    };
    if r.get(at + 4) != Some(&0) || r.get(at + 5) != Some(&0x83) {
        return None;
    }
    Some(((*r.get(at + 7)?).min(100), *r.get(at + 6)? != 0))
}
pub fn status_request(id: u8, len: usize) -> Vec<u8> {
    let mut out = vec![0xff; len];
    out[0] = id;
    out[1] = !0x06;
    out
}
pub fn parse_status(r: &[u8], vid: u16) -> Option<(u8, bool)> {
    if r.len() < 13 || !matches!(r[0], 0x11 | 0x12) || r[1] != !0x06 {
        return None;
    }
    let actual = u16::from_le_bytes([!r[2], !r[3]]);
    let level = !r[11];
    let model = u16::from_le_bytes([!r[4], !r[5]]);
    (actual == vid && (vid != 0x5253 || model == 0x0031) && level <= 100)
        .then_some((level, !r[12] != 0))
}
pub fn parse_g7(r: &[u8]) -> Option<(u8, bool)> {
    (r.len() >= 10 && r[..2] == [0xaa, 0x30] && r[8] <= 100).then_some((r[8], r[9] != 0))
}
fn query(model: Model, i: &hid::Interface) -> Result<Option<(u8, bool)>, String> {
    match model {
        Model::A5 => {
            let dev = hid::Device::open(&i.path)?;
            let req = a5_request();
            dev.send(&req[..i.feature_len.min(65) as usize])?;
            std::thread::sleep(Duration::from_millis(34));
            for attempt in 0..8 {
                if attempt == 3 {
                    dev.send(&req[..i.feature_len.min(65) as usize])?;
                    std::thread::sleep(Duration::from_millis(34));
                }
                if let Ok(reply) = dev.receive(0, i.feature_len as usize) {
                    if let Some(result) = parse_a5(&reply) {
                        return Ok(Some(result));
                    }
                }
                std::thread::sleep(Duration::from_millis(30));
            }
        }
        Model::G7 => {
            let mut transport = WinTransport::open(&i.path)?;
            transport.flush()?;
            let mut req = [0; 65];
            req[..9].copy_from_slice(&[0, 0x55, 0x30, 0xa5, 0x0b, 0x2e, 1, 1, 1]);
            transport.write(&req, Duration::from_millis(400))?;
            for _ in 0..25 {
                let reply = transport.read(i.input_len as usize, Duration::from_millis(20))?;
                if let Some(result) = parse_g7(&reply) {
                    return Ok(Some(result));
                }
            }
        }
        Model::A7 | Model::M7 => {
            let dev = hid::Device::open(&i.path)?;
            for (id, len) in [(0x11, 21), (0x12, 65)] {
                if i.feature_len < len {
                    continue;
                }
                let req = status_request(id, len as usize);
                for _ in 0..4 {
                    dev.send(&req)?;
                    std::thread::sleep(Duration::from_millis(120));
                    if let Ok(reply) = dev.receive(id, len as usize) {
                        if let Some(result) = parse_status(&reply, i.vid) {
                            return Ok(Some(result));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}
pub fn read_state(model: Model) -> Result<State, String> {
    let (key, name, vid, _) = model.spec();
    let mut candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| model.matches(i))
        .collect();
    candidates.sort_by_key(|i| (i.pid != 0x0031 && i.pid != 0xf019, i.pid));
    let mut first_error = None;
    for i in &candidates {
        match query(model, i) {
            Ok(Some((percent, charging))) => {
                return Ok(State {
                    key: key.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    wired: matches!(model, Model::A5) && i.pid == 0xf019
                        || matches!(model, Model::M7) && i.pid == 0x0031,
                    identity: format!("{name} · {vid:04X}:{:04X}", i.pid),
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
        key: key.into(),
        present: !candidates.is_empty(),
        error: first_error.unwrap_or_else(|| format!("{name} 未取得有效电量回复（设备可能休眠）")),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames() {
        assert_eq!(&a5_request()[..7], &[0, 0, 0, 2, 2, 0, 0x83]);
        assert_eq!(
            parse_a5(&[0xa1, 0, 0, 0, 0, 0x83, 1, 120, 0]),
            Some((100, true))
        );
        assert_eq!(
            parse_g7(&[0xaa, 0x30, 0xa5, 0x3c, 0x0a, 1, 1, 1, 46, 1]),
            Some((46, true))
        );
        let mut r = vec![0; 21];
        r[0] = 0x11;
        r[1] = 0xf9;
        r[2] = !0x53;
        r[3] = !0x52;
        r[4] = !0x31;
        r[5] = !0x00;
        r[11] = !64;
        r[12] = !1;
        assert_eq!(parse_status(&r, 0x5253), Some((64, true)));
        r[4] = !0x32;
        assert_eq!(parse_status(&r, 0x5253), None);
    }
}
