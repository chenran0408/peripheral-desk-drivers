//! Passive DualShock 4 USB/Bluetooth input reports, based on HaloBattery providers/playstation.py.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn parse(frame: &[u8]) -> Option<(u8, bool, bool)> {
    let (index, wired) = match frame.first()? {
        0x01 if frame.len() > 30 => (30, true),
        0x11 if frame.len() > 32 => (32, false),
        _ => return None,
    };
    let status = frame[index];
    let level = status & 0x0f;
    (level <= 11).then_some((level.min(10) * 10, status & 0x10 != 0, wired))
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let i = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x054c
                && i.pid == 0x09cc
                && i.usage_page == 1
                && i.usage == 5
                && i.input_len > 32
        })
        .ok_or("未找到 DualShock 4 输入接口")?;
    let mut io = WinTransport::open(&i.path)?;
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        if let Some((percent, charging, wired)) =
            parse(&io.read(i.input_len as usize, Duration::from_millis(250))?)
        {
            return Ok(State {
                key: "sony-dualshock-4".into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired,
                identity: format!(
                    "Sony DualShock 4 · 054C:09CC · {}",
                    if wired { "USB" } else { "Bluetooth" }
                ),
                settings_error: "仅被动监听电量输入报告。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            });
        }
    }
    Err("DualShock 4 未发送有效电量输入报告".into())
}
