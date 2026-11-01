//! Bluetooth battery header and device-info query from HaloBattery's Nintendo provider (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "nintendo-switch-pro";
pub const PIDS: &[u16] = &[0x2009, 0x2006, 0x2007];
const BT_GUID: &str = "{00001124-0000-1000-8000-00805f9b34fb}";

pub fn parse(frame: &[u8]) -> Option<(u8, bool)> {
    if frame.len() < 3 || !matches!(frame[0], 0x21 | 0x30 | 0x31) {
        return None;
    }
    let level = (frame[2] & 0xe0) >> 4;
    (level <= 8).then_some(((level as u16 * 100 / 8) as u8, frame[2] & 0x10 != 0))
}

pub fn device_info_request() -> [u8; 49] {
    let mut frame = [0; 49];
    frame[..11].copy_from_slice(&[1, 0, 0, 1, 0x40, 0x40, 0, 1, 0x40, 0x40, 2]);
    frame
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let iface = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x057e
                && PIDS.contains(&i.pid)
                && i.input_len >= 3
                && (i.path.to_ascii_lowercase().contains(BT_GUID)
                    || i.path.to_ascii_lowercase().contains("vid&"))
                && i.usage_page == 1
                && matches!(i.usage, 4 | 5)
        })
        .ok_or("未找到 Switch 蓝牙手柄输入接口")?;
    let mut io = WinTransport::open(&iface.path)?;
    let mut battery = None;
    let listen_end = Instant::now() + Duration::from_millis(100);
    while Instant::now() < listen_end {
        let frame = io.read(iface.input_len as usize, Duration::from_millis(25))?;
        if matches!(frame.first(), Some(0x30 | 0x31)) {
            battery = parse(&frame);
            if battery.is_some() {
                break;
            }
        }
    }
    if battery.is_none() {
        io.write(&device_info_request(), Duration::from_millis(500))?;
        let end = Instant::now() + Duration::from_millis(600);
        while Instant::now() < end {
            let frame = io.read(iface.input_len as usize, Duration::from_millis(50))?;
            if frame.first() == Some(&0x21) {
                battery = parse(&frame);
                if battery.is_some() {
                    break;
                }
            }
        }
    }
    let (percent, charging) = battery.ok_or("Switch 手柄未回复有效电量")?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: Some(charging),
        identity: format!(
            "Nintendo Switch Pro / Joy-Con · 057E:{:04X} · Bluetooth",
            iface.pid
        ),
        settings_error: "仅查询设备信息，不改变手柄模式。".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_and_query() {
        assert_eq!(parse(&[0x30, 0, 0x50]), Some((50, true)));
        assert_eq!(parse(&[0x21, 0, 0xe0]), None);
        assert_eq!(parse(&[0x3f, 0, 0x80]), None);
        assert_eq!(
            &device_info_request()[..11],
            &[1, 0, 0, 1, 0x40, 0x40, 0, 1, 0x40, 0x40, 2]
        );
    }
}
