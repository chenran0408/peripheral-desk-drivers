//! Passive JBL Quantum 910 reports from plugato/JBL_Baterry_Monitor and HaloBattery (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const KEY: &str = "jbl-quantum-910";
static CACHE: OnceLock<Mutex<Option<(String, u8, u64)>>> = OnceLock::new();

pub fn parse(reply: &[u8]) -> Option<u8> {
    (reply.len() >= 2 && reply[0] == 8 && reply[1] <= 100).then(|| reply[1])
}
fn candidate(i: &hid::Interface) -> bool {
    i.vid == 0x0ecb
        && i.pid == 0x2088
        && i.usage_page == 0xff13
        && i.usage == 1
        && i.input_len as usize >= 2
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let Some(i) = interfaces.iter().find(|i| candidate(i)) else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 JBL Quantum 910 接收器".into(),
            ..State::default()
        });
    };
    let mut transport = WinTransport::open(&i.path)?;
    let end = Instant::now() + Duration::from_millis(500);
    let mut newest = None;
    while Instant::now() < end {
        let report = transport.read(i.input_len as usize, Duration::from_millis(100))?;
        if let Some(level) = parse(&report) {
            newest = Some(level);
        }
    }
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut last = cache.lock().map_err(|e| e.to_string())?;
    if let Some(level) = newest {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        *last = Some((i.path.clone(), level, now));
    }
    let percent = newest.or_else(|| {
        last.as_ref()
            .and_then(|(path, value, _)| (path == &i.path).then_some(*value))
    });
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: newest.is_some(),
        percent,
        charging: None,
        identity: "JBL Quantum 910 Wireless · 0ECB:2088 · FF13:0001".into(),
        updated: last
            .as_ref()
            .and_then(|(path, _, time)| (path == &i.path).then_some(*time))
            .unwrap_or(0),
        settings_error: "被动监听；静默时显示缓存电量，开关机事件不作为电量。".into(),
        ..State::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn battery_report_only() {
        assert_eq!(parse(&[8, 95]), Some(95));
        assert_eq!(parse(&[9, 1]), None);
        assert_eq!(parse(&[8, 101]), None);
    }
}
