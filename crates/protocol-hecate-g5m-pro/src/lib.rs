//! HECATE G5M Pro receiver battery reader, adapted from Vigil Watt v1.3.0.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "hecate-g5m-pro";
const VID: u16 = 0x35BB;
const PID: u16 = 0xD00B;
const USAGE_PAGE: u16 = 0xFF1C;
const USAGE: u16 = 0x0092;
const REPORT: usize = 64;
const ID: u8 = 0x04;
const MARKER: u8 = 0x1A;

pub fn query_frame() -> [u8; REPORT] {
    let mut f = [0; REPORT];
    f[0] = ID;
    f[3] = MARKER;
    f
}
pub fn parse_status(frame: &[u8]) -> Option<(u8, Option<bool>)> {
    if frame.len() != REPORT || frame[0] != ID || frame[3] != MARKER {
        return None;
    }
    let level = frame[8];
    if !(1..=100).contains(&level) {
        return None;
    }
    let charging = match frame[9] {
        0x00 => Some(false),
        0x01 => Some(true),
        0x02 => Some(true),
        _ => None,
    };
    Some((level, charging))
}
fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT
        && i.output_len as usize == REPORT
        && i.feature_len == 0
}
pub fn read_state() -> Result<State, String> {
    let items = hid::enumerate()?;
    let mut found = items.iter().filter(|i| supported(i));
    let Some(i) = found.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "未找到 HECATE G5M Pro 指定接收器接口".into(),
            settings_error: "仅支持 2.4 GHz 接收器电量与充电状态；其他控制未适配。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 G5M Pro 适配接口，已停止查询".into());
    }
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    io.write(&query_frame(), Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(1200);
    while Instant::now() < deadline {
        let frame = io.read(REPORT, Duration::from_millis(200))?;
        if frame.is_empty() {
            continue;
        }
        if let Some((percent, charging)) = parse_status(&frame) {
            return Ok(State {
                key: DEVICE_KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging,
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                identity: "HECATE G5M Pro · 2.4 GHz 接收器 · 35BB:D00B · FF1C:0092".into(),
                settings_error: "只读电量与充电状态；DPI、改键、宏、灯效和音效未适配。".into(),
                ..State::default()
            });
        }
    }
    Err("G5M Pro 未收到本轮匹配的 0x04 状态帧".into())
}
