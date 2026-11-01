//! Keychron status exchange, based on csutcliff/keychron-battery-dkms (GPL; protocol facts only).
use peripheral_desk::{
    hid::{self, Device},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn parse(frame: &[u8]) -> Option<u8> {
    (frame.len() >= 21 && frame[..2] == [0xb4, 0x06] && frame[20] <= 100).then_some(frame[20])
}

pub fn read_state(key: &str, name: &str, pid: u16) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let interface = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x3434
                && i.pid == pid
                && i.path.to_ascii_lowercase().contains("mi_04")
                && i.input_len >= 64
                && i.feature_len >= 64
        })
        .ok_or(format!("未找到 Keychron {name} 接口 4"))?;
    let mut io = WinTransport::open(&interface.path)?;
    let feature = Device::open(&interface.path)?;
    let mut request = [0; 64];
    request[..2].copy_from_slice(&[0xb3, 0x06]);
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }
        io.flush()?;
        feature.send(&request)?;
        if let Some(percent) = parse(&io.read(64, Duration::from_millis(500))?) {
            return Ok(State {
                key: key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                wired: pid == 0xd048,
                identity: format!("Keychron {name} · 3434:{pid:04X} · MI_04"),
                settings_error: "仅提供只读电量。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            });
        }
    }
    Err("Keychron 三次状态查询均未收到有效电量".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status() {
        let mut reply = [0; 64];
        reply[..2].copy_from_slice(&[0xb4, 6]);
        reply[20] = 87;
        assert_eq!(parse(&reply), Some(87));
        reply[20] = 101;
        assert_eq!(parse(&reply), None);
    }
}
