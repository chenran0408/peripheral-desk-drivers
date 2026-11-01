//! Corsair headset exchanges from HaloBattery providers/corsair.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const VIRTUOSO_KEY: &str = "corsair-virtuoso-rgb-se-xt";
pub const VOID_KEY: &str = "corsair-void-v2-family";
const LEN: usize = 64;

pub fn virtuoso_request(target: u8, property: u8) -> [u8; LEN] {
    let mut data = [0; LEN];
    data[..4].copy_from_slice(&[2, target, 2, property]);
    data
}

pub fn parse_virtuoso(reply: &[u8]) -> Option<u32> {
    (reply.len() >= 8 && reply[..4] == [1, 1, 2, 0])
        .then(|| u32::from_le_bytes(reply[4..8].try_into().unwrap()))
}

pub fn void_request(endpoint: u8, command: u8) -> [u8; 65] {
    let mut data = [0; 65];
    data[..5].copy_from_slice(&[0, 2, endpoint, 2, command]);
    data
}

pub fn parse_void(reply: &[u8]) -> Option<u8> {
    if reply.len() < 6 {
        return None;
    }
    let raw = u16::from_le_bytes([reply[4], reply[5]]);
    (raw > 0 && raw <= 1000).then_some((raw / 10) as u8)
}

fn base(key: &str, i: &hid::Interface, percent: u8, charging: Option<bool>) -> State {
    State {
        key: key.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging,
        wired: matches!(i.pid, 0x0a3d | 0x0a62),
        identity: format!("Corsair · 1B1C:{:04X}", i.pid),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅支持只读电量与充电状态。".into(),
        ..State::default()
    }
}

fn ask_virtuoso(
    io: &mut WinTransport,
    target: u8,
    prop: u8,
    window: Duration,
) -> Result<Option<u32>, String> {
    io.write(&virtuoso_request(target, prop), Duration::from_millis(500))?;
    let end = Instant::now() + window;
    while Instant::now() < end {
        let r = io.read(LEN, Duration::from_millis(150))?;
        if r.first() == Some(&0x0e) {
            continue;
        }
        if let Some(v) = parse_virtuoso(&r) {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

pub fn read_virtuoso() -> Result<State, String> {
    let mut interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x1b1c
                && matches!(i.pid, 0x0a3e | 0x0a3d | 0x0a40 | 0x0a64 | 0x0a62)
                && i.usage_page == 0xff42
                && i.usage == 1
        })
        .collect();
    interfaces.sort_by_key(|i| (i.pid != 0x0a3d && i.pid != 0x0a62, i.usage_page != 0xff42));
    if interfaces.is_empty() {
        return Ok(State {
            key: VIRTUOSO_KEY.into(),
            error: "未找到 Virtuoso 控制接口".into(),
            ..State::default()
        });
    }
    let mut best = None;
    for i in &interfaces {
        let Ok(mut io) = WinTransport::open(&i.path) else {
            continue;
        };
        if io.flush().is_err() {
            continue;
        }
        let targets = if matches!(i.pid, 0x0a3d | 0x0a62) {
            [8, 9]
        } else {
            [9, 8]
        };
        if i.pid == 0x0a40 {
            for (route, prop) in [(8, 0x13), (8, 0x12), (9, 0x12)] {
                let _ = ask_virtuoso(&mut io, route, prop, Duration::from_millis(400));
            }
        }
        for target in targets {
            let Ok(Some(raw)) = ask_virtuoso(&mut io, target, 0x0f, Duration::from_millis(1600))
            else {
                continue;
            };
            if raw == 0 || raw > 1000 {
                continue;
            }
            // The reply echoes the command, not the property. A second answer
            // distinguishes battery data from another app's property read.
            if let Ok(Some(confirm)) =
                ask_virtuoso(&mut io, target, 0x0f, Duration::from_millis(1200))
            {
                if raw != confirm {
                    continue;
                }
            }
            let charging = ask_virtuoso(&mut io, target, 0x10, Duration::from_millis(1200))
                .ok()
                .flatten()
                .and_then(|v| match v {
                    1 => Some(true),
                    2 => Some(false),
                    _ => None,
                });
            let state = base(VIRTUOSO_KEY, i, (raw / 10) as u8, charging);
            if charging == Some(true) {
                return Ok(state);
            }
            if best.is_none() {
                best = Some(state);
            }
            break;
        }
    }
    best.ok_or_else(|| "Virtuoso 未返回有效电量".into())
}

pub fn read_void() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x1b1c
                && matches!(i.pid, 0x2a08 | 0x2a02 | 0x0a97)
                && i.path.to_ascii_lowercase().contains("mi_04")
                && i.usage_page >= 0xff00
        })
        .collect();
    if interfaces.is_empty() {
        return Ok(State {
            key: VOID_KEY.into(),
            error: "未找到 Corsair 接收器接口 4".into(),
            ..State::default()
        });
    }
    for i in &interfaces {
        let Ok(mut io) = WinTransport::open(&i.path) else {
            continue;
        };
        io.flush()?;
        io.write(&void_request(8, 0x13), Duration::from_millis(500))?;
        io.write(&void_request(8, 0x12), Duration::from_millis(500))?;
        io.flush()?;
        io.write(&void_request(9, 0x12), Duration::from_millis(500))?;
        if io.read(LEN, Duration::from_millis(500))?.is_empty() {
            continue;
        }
        io.flush()?;
        for _ in 0..3 {
            io.write(&void_request(9, 0x0f), Duration::from_millis(500))?;
            if let Some(percent) = parse_void(&io.read(LEN, Duration::from_millis(500))?) {
                return Ok(base(VOID_KEY, i, percent, None));
            }
        }
    }
    Err("Corsair 耳机未返回有效电量".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames() {
        assert_eq!(&virtuoso_request(9, 15)[..4], &[2, 9, 2, 15]);
        assert_eq!(parse_virtuoso(&[1, 1, 2, 0, 0xca, 3, 0, 0]), Some(970));
        assert_eq!(parse_virtuoso(&[0x0e, 1, 2, 0, 0xca, 3, 0, 0]), None);
        assert_eq!(&void_request(9, 15)[..5], &[0, 2, 9, 2, 15]);
        assert_eq!(parse_void(&[0, 0, 0, 0, 0xe8, 3]), Some(100));
    }
}
