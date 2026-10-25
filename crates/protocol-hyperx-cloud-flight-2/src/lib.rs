//! HyperX Cloud Flight 2 passive battery/charging listener from Vigil Watt v1.3.0.
//! Exact scope: HP 03F0:0AC1, status interface MI_03&COL04, usage page FF90.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::{
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const DEVICE_KEY: &str = "hyperx-cloud-flight-2";
const VID: u16 = 0x03F0;
const PID: u16 = 0x0AC1;
const USAGE_PAGE: u16 = 0xFF90;
const USAGE: u16 = 0xFF00;
const REPORT_LEN: usize = 64;

#[derive(Default)]
struct Snapshot {
    online: bool,
    percent: Option<u8>,
    charging: Option<bool>,
    seen: u64,
    battery_at: u64,
}
static SNAPSHOT: OnceLock<Mutex<Snapshot>> = OnceLock::new();
static LISTENER: OnceLock<()> = OnceLock::new();
fn cache() -> &'static Mutex<Snapshot> {
    SNAPSHOT.get_or_init(|| Mutex::new(Snapshot::default()))
}

/// Returns (online event, battery percent, charging state) from known event formats only.
pub fn parse_frame(frame: &[u8]) -> Option<(Option<bool>, Option<u8>, Option<bool>)> {
    if frame.len() < 4 {
        return None;
    }
    if frame[0] == 0x45 && frame[1] == 0x02 {
        return Some((Some(true), None, None));
    }
    if frame[0] == 0xFB {
        return match (frame[1], frame[2]) {
            (0x0A, 0x00) => Some((Some(false), None, None)),
            (0x0A, 0x01) => Some((Some(true), None, None)),
            (0x09, 0x00) => Some((None, None, Some(false))),
            (0x09, 0x01) => Some((None, None, Some(true))),
            (0x13, _) if frame[3] <= 100 => Some((None, Some(frame[3]), None)),
            _ => None,
        };
    }
    // Upstream observed this initial/reconnect frame as a full battery snapshot.
    if frame[0] == 0x51 && frame[1] == 0x02 && frame[2] <= 100 {
        return Some((Some(true), Some(frame[2]), None));
    }
    None
}

fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&mi_03&col04#")
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn start_listener() {
    LISTENER.get_or_init(|| {
        thread::Builder::new()
            .name("hyperx-cf2-status".into())
            .spawn(|| loop {
                let interface = hid::enumerate()
                    .ok()
                    .and_then(|items| items.into_iter().find(supported));
                let Some(interface) = interface else {
                    thread::sleep(Duration::from_secs(2));
                    continue;
                };
                let Ok(mut transport) = WinTransport::open(&interface.path) else {
                    thread::sleep(Duration::from_secs(2));
                    continue;
                };
                loop {
                    match transport.read(REPORT_LEN, Duration::from_secs(2)) {
                        Ok(frame) if !frame.is_empty() => {
                            if let Some((online, percent, charging)) = parse_frame(&frame) {
                                if let Ok(mut state) = cache().lock() {
                                    state.seen = now();
                                    if let Some(value) = online {
                                        state.online = value;
                                        if !value {
                                            state.percent = None;
                                            state.charging = None;
                                        }
                                    }
                                    if let Some(value) = percent {
                                        state.percent = Some(value);
                                        state.battery_at = state.seen;
                                    }
                                    if let Some(value) = charging {
                                        state.charging = Some(value);
                                    }
                                }
                            }
                        }
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
            })
            .ok();
    });
}

pub fn read_state() -> Result<State, String> {
    start_listener();
    let state = cache()
        .lock()
        .map_err(|_| "HyperX 状态缓存暂不可用".to_string())?;
    let timestamp = now();
    let present = state.online && timestamp.saturating_sub(state.seen) <= 5;
    let fresh = state.percent.is_some() && timestamp.saturating_sub(state.battery_at) <= 180;
    let percent = if fresh { state.percent } else { None };
    Ok(State {
        key: DEVICE_KEY.into(),
        present,
        fresh,
        percent,
        charging: state.charging,
        wired: false,
        updated: state.battery_at,
        identity: "HyperX Cloud Flight 2 · USB 接收器 · 03F0:0AC1 · FF90 状态流".into(),
        error: if percent.is_none() {
            "等待耳机推送电量帧；该协议不主动查询".into()
        } else {
            String::new()
        },
        settings_error: "只读被动电量/充电状态；不提供 EQ、灯光或控制写入。无需主动发送 HID 命令。"
            .into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_only_known_event_offsets() {
        assert_eq!(
            parse_frame(&[0xFB, 0x13, 0x00, 73]),
            Some((None, Some(73), None))
        );
        assert_eq!(
            parse_frame(&[0xFB, 0x09, 0x01, 0]),
            Some((None, None, Some(true)))
        );
        assert_eq!(
            parse_frame(&[0xFB, 0x0A, 0x00, 0]),
            Some((Some(false), None, None))
        );
        assert_eq!(parse_frame(&[0xFB, 0x13, 0x00, 101]), None);
        assert_eq!(parse_frame(&[0xFB, 0x0A, 0x02, 0]), None);
        assert_eq!(
            parse_frame(&[0x45, 0x02, 0, 0]),
            Some((Some(true), None, None))
        );
    }
}
