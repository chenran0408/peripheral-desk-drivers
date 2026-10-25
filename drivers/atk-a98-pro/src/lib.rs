//! ATK A98 Pro VIA battery channel, based on Vigil Watt v1.3.0 (a78e973).
//! Independently implemented; no Vigil Watt source code is copied.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "atk-a98-pro";
const VID: u16 = 0x373b;
const PID: u16 = 0x1152;
const PAGE: u16 = 0xff60;
const USAGE: u16 = 0x0061;
const REPORT_ID: u8 = 0;
const REPORT_LEN: usize = 33;

fn matches(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
}

fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    for off in [1usize, 0] {
        if reply.len() < off + 5 {
            continue;
        }
        let frame = &reply[off..];
        if frame[0] != 0x02
            || frame[1] != 0x96
            || frame[2] != 0x24
            || frame[3] == 0
            || frame[3] > 100
        {
            continue;
        }
        return Some((frame[3], frame[4] != 0));
    }
    None
}

fn read_state() -> Result<State, String> {
    let ports = hid::enumerate()?;
    let found: Vec<_> = ports.iter().filter(|i| matches(i)).collect();
    if found.len() != 1 {
        return Err("未找到唯一的 ATK A98 Pro VIA 电量接口".into());
    }
    let port = found[0];
    let mut wire = [0; REPORT_LEN];
    wire[0] = REPORT_ID;
    wire[1..4].copy_from_slice(&[0x02, 0x96, 0x24]);
    let mut io = WinTransport::open(&port.path)?;
    io.flush()?;
    io.write(&wire, Duration::from_millis(500))?;
    let until = Instant::now() + Duration::from_secs(1);
    let (percent, charging) = loop {
        if Instant::now() >= until {
            return Err("ATK A98 Pro 电量查询未收到有效 VIA 回复".into());
        }
        let reply = io.read(REPORT_LEN, Duration::from_millis(100))?;
        if let Some(value) = parse(&reply) {
            break value;
        }
    };
    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(percent);
    state.charging = Some(charging);
    state.wired = false;
    state.identity = "ATK A98 Pro · VIA 接收器".into();
    state.updated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(state)
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("ATK A98 Pro 插件当前提供电量与充电状态；VIA 改键、宏和灯效控制待集成".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("atk-a98-pro", handler);
