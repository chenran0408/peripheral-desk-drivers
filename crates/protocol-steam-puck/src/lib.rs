//! Passive puck slot listener, based on SDL Triton and HaloBattery providers/steam.py.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static LAST: OnceLock<Mutex<HashMap<String, (u8, bool)>>> = OnceLock::new();

pub fn parse(frame: &[u8]) -> Option<(u8, bool)> {
    if frame.len() < 15 || frame[0] != 0x43 || frame[2] > 100 {
        return None;
    }
    let charging = match frame[1] {
        1 => false,
        2 | 4 => true,
        _ => return None,
    };
    Some((frame[2], charging))
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let slots: Vec<_> = interfaces
        .iter()
        .filter(|i| {
            i.vid == 0x28de
                && matches!(i.pid, 0x1304 | 0x1305)
                && i.usage_page == 0xff00
                && i.usage == 1
                && (2..=5).any(|n| i.path.to_ascii_lowercase().contains(&format!("mi_{n:02x}")))
        })
        .collect();
    let cache = LAST.get_or_init(|| Mutex::new(HashMap::new()));
    let mut last = cache.lock().map_err(|e| e.to_string())?;
    if slots.is_empty() {
        last.clear();
        return Err("未找到 Steam Controller 2025 puck 槽位".into());
    }
    let mut selected = None;
    for i in slots {
        let mut io = WinTransport::open(&i.path)?;
        let short = Instant::now() + Duration::from_millis(400);
        let long = Instant::now() + Duration::from_secs(5);
        let mut awake = false;
        let key = i.path.clone();
        while Instant::now() < if awake { long } else { short } {
            let report = io.read(i.input_len as usize, Duration::from_millis(50))?;
            match report.first() {
                Some(0x43) => {
                    if let Some(reading) = parse(&report) {
                        last.insert(key.clone(), reading);
                        selected = Some((i.pid, true, reading));
                    }
                    awake = true;
                    if selected.is_some() {
                        break;
                    }
                }
                Some(0x42 | 0x45 | 0x47) => awake = true,
                Some(0x46 | 0x79) if report.get(1) == Some(&1) => {
                    last.remove(&key);
                    break;
                }
                Some(0x46 | 0x79) if report.get(1) == Some(&2) => awake = true,
                _ => {}
            }
        }
        if selected.is_none() {
            if let Some(reading) = last.get(&key).copied() {
                selected = Some((i.pid, false, reading));
            }
        }
        if selected.as_ref().is_some_and(|(_, fresh, _)| *fresh) {
            break;
        }
    }
    let (pid, fresh, (percent, charging)) = selected.ok_or("Steam puck 尚无电量输入报告")?;
    Ok(State {
        key: "steam-controller-2025".into(),
        present: true,
        fresh,
        percent: Some(percent),
        charging: Some(charging),
        identity: format!("Steam Controller 2025 · 28DE:{pid:04X} · puck"),
        settings_error: "仅被动监听；静默槽保留上次电量并显示为过期读数。".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    })
}
