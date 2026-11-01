//! Passive D-input battery report from HaloBattery providers/eightbitdo.py and SDL.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn parse(frame: &[u8]) -> Option<(u8, bool)> {
    if frame.len() <= 14 || frame[0] != 0x04 {
        return None;
    }
    let percent = frame[14] & 0x7f;
    (percent <= 100).then_some((percent, frame[14] & 0x80 != 0))
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let i = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x2dc8
                && i.pid == 0x6009
                && i.input_len >= 15
                && i.path
                    .to_ascii_lowercase()
                    .contains("{00001124-0000-1000-8000-00805f9b34fb}")
        })
        .ok_or("未找到 8BitDo Pro 3 D-input 输入接口")?;
    let mut io = WinTransport::open(&i.path)?;
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        if let Some((percent, charging)) =
            parse(&io.read(i.input_len as usize, Duration::from_millis(250))?)
        {
            return Ok(State {
                key: "8bitdo-pro-3".into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: false,
                identity: "8BitDo Pro 3 · D-input · 2DC8:6009".into(),
                settings_error: "仅被动监听输入报告，不向手柄发送命令。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            });
        }
    }
    Err("未收到 Pro 3 的 0x04 电量输入报告".into())
}
