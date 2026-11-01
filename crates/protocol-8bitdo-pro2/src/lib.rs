//! Passive D-input report documented by HaloBattery and SDL; never switches report mode.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "eightbitdo-pro-2";
pub const PIDS: &[u16] = &[0x6000, 0x6100, 0x6001, 0x6101, 0x6003, 0x6006];
const BT_GUID: &str = "{00001124-0000-1000-8000-00805f9b34fb}";

pub fn parse(frame: &[u8]) -> Option<(u8, bool)> {
    if frame.len() <= 14 || frame[0] != 1 {
        return None;
    }
    let percent = frame[14] & 0x7f;
    (1..=100)
        .contains(&percent)
        .then_some((percent, frame[14] & 0x80 != 0))
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let iface = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x2dc8
                && PIDS.contains(&i.pid)
                && i.input_len > 14
                && (i.path.to_ascii_lowercase().contains(BT_GUID)
                    || i.path.to_ascii_lowercase().contains("vid&"))
                && i.usage_page == 1
                && matches!(i.usage, 4 | 5)
        })
        .ok_or("未找到 8BitDo D-input 蓝牙输入接口")?;
    let mut io = WinTransport::open(&iface.path)?;
    let end = Instant::now() + Duration::from_millis(500);
    while Instant::now() < end {
        if let Some((percent, charging)) =
            parse(&io.read(iface.input_len as usize, Duration::from_millis(50))?)
        {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: false,
                identity: format!(
                    "8BitDo Pro 2 / SN30 Pro / SF30 Pro · 2DC8:{:04X}",
                    iface.pid
                ),
                settings_error: "仅被动监听电量；Steam 或游戏启用增强输入报告时显示。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            });
        }
    }
    Ok(State {
        key: KEY.into(),
        present: true,
        identity: format!("8BitDo D-input · 2DC8:{:04X}", iface.pid),
        error: "未收到增强输入报告；需由 Steam 或游戏启用".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enhanced_only() {
        let mut frame = [0; 20];
        frame[0] = 1;
        assert_eq!(parse(&frame), None);
        frame[14] = 0x80 | 57;
        assert_eq!(parse(&frame), Some((57, true)));
        frame[14] = 101;
        assert_eq!(parse(&frame), None);
    }
}
