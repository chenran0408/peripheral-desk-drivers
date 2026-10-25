//! Razer Kraken V4 receiver battery and charging queries, adapted from Vigil Watt v1.3.0.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Mutex, OnceLock,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "razer-kraken-v4";
const VID: u16 = 0x1532;
const PID: u16 = 0x056C;
const USAGE_PAGE: u16 = 0xFF14;
const USAGE: u16 = 0x0001;
const REPORT_ID: u8 = 0x02;
const REPORT_LEN: usize = 64;
const BATTERY: u8 = 0x21;
const CHARGING: u8 = 0x2A;
type LastSnapshot = (Option<u8>, Option<bool>, u64, u64);
static NEXT: AtomicU8 = AtomicU8::new(0);
static LAST: OnceLock<Mutex<LastSnapshot>> = OnceLock::new();

pub fn build_query(command: u8) -> Option<[u8; REPORT_LEN]> {
    if command != BATTERY && command != CHARGING {
        return None;
    }
    let mut f = [0u8; REPORT_LEN];
    f[0] = REPORT_ID;
    f[2] = 0x60;
    f[6] = 4;
    f[9] = 0x80;
    f[10] = command;
    f[11] = 0;
    let check = f[..REPORT_LEN - 2].iter().fold(0, |acc, b| acc ^ b);
    f[REPORT_LEN - 2] = check;
    Some(f)
}
fn parse_reply(f: &[u8], command: u8) -> Option<u8> {
    if f.len() != REPORT_LEN
        || f[0] != REPORT_ID
        || f[1] != 2
        || f[2] != 0x60
        || f[10] != command
        || f[11] != 1
        || f[12] != 1
    {
        return None;
    }
    let checksum = f[1..REPORT_LEN - 2].iter().fold(0, |acc, b| acc ^ b);
    (f[REPORT_LEN - 2] == checksum).then_some(f[13])
}
pub fn parse_battery(f: &[u8]) -> Option<u8> {
    let p = parse_reply(f, BATTERY)?;
    (p <= 100).then_some(p)
}
pub fn parse_charging(f: &[u8]) -> Option<bool> {
    let v = parse_reply(f, CHARGING)?;
    (v <= 1).then_some(v == 1)
}
fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
}
fn query(path: &str, cmd: u8) -> Result<u8, String> {
    let mut io = WinTransport::open(path)?;
    io.write(
        &build_query(cmd).ok_or("未知 Kraken V4 查询命令")?,
        Duration::from_millis(500),
    )?;
    let until = Instant::now() + Duration::from_millis(1200);
    while Instant::now() < until {
        let frame = io.read(REPORT_LEN, Duration::from_millis(200))?;
        if frame.is_empty() {
            continue;
        }
        let value = if cmd == BATTERY {
            parse_battery(&frame)
        } else {
            parse_charging(&frame).map(u8::from)
        };
        if let Some(value) = value {
            return Ok(value);
        }
    }
    Err("Kraken V4 未收到匹配的查询回复".into())
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(i) = found.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "未找到 Kraken V4 接收器电量接口".into(),
            settings_error: "只读电量和充电查询；DPI/灯效/音效未适配。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 Kraken V4 匹配接口，已停止查询".into());
    }
    // Receiver frame-limits consecutive queries; alternate one command each host poll.
    let turn = NEXT.fetch_xor(1, Ordering::Relaxed);
    let command = if turn == 0 { BATTERY } else { CHARGING };
    let value = query(&i.path, command)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cache = LAST.get_or_init(|| Mutex::new((None, None, 0, 0)));
    let mut cached = cache
        .lock()
        .map_err(|_| "Kraken V4 状态缓存不可用".to_string())?;
    if command == BATTERY {
        cached.0 = Some(value);
        cached.2 = now;
    } else {
        cached.1 = Some(value == 1);
        cached.3 = now;
    }
    let percent = (now.saturating_sub(cached.2) <= 180)
        .then_some(cached.0)
        .flatten();
    let charging = (now.saturating_sub(cached.3) <= 180)
        .then_some(cached.1)
        .flatten();
    Ok(State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: percent.is_some() && charging.is_some(),
        percent,
        charging,
        updated: now,
        identity: "Razer Kraken V4 · USB 接收器 · 1532:056C · report 02".into(),
        settings_error: "只读电量与充电状态；该查询插件不包含音效、EQ、灯效或控制。".into(),
        ..State::default()
    })
}
