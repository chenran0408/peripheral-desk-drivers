//! VGN V98 Pro battery adapter from Vigil Watt v1.3.0 (a78e973).
//! Independently implemented from documented frames; no source is copied.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "vgn-v98-pro";
const VID: u16 = 0x320f;
const PID: u16 = 0x5088;
const PAGE: u16 = 0xff1c;
const USAGE: u16 = 0x0092;
const REPORT_ID: u8 = 0x04;

fn matches(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.output_len >= 8
        && i.input_len >= 10
}

fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    if reply.len() <= 9 || reply[0] != REPORT_ID || reply[3] != 0x1a || reply[8] > 100 {
        return None;
    }
    Some((reply[8], reply[9] != 0))
}

fn read_state() -> Result<State, String> {
    let ports = hid::enumerate()?;
    let found: Vec<_> = ports.iter().filter(|i| matches(i)).collect();
    if found.len() != 1 {
        return Err("未找到唯一的 VGN V98 Pro 接收器接口".into());
    }
    let port = found[0];
    let mut request = vec![0; port.output_len as usize];
    if request.len() < 8 || request.len() > 4096 {
        return Err("VGN 输出报告长度不匹配".into());
    }
    request[0] = REPORT_ID;
    request[1..8].copy_from_slice(&[0, 0, 0x1a, 0x06, 0, 0, 0]);
    let mut io = WinTransport::open(&port.path)?;
    io.flush()?;
    io.write(&request, Duration::from_millis(500))?;
    let until = Instant::now() + Duration::from_secs(1);
    let (percent, charging) = loop {
        if Instant::now() >= until {
            return Err("VGN V98 Pro 电量查询未收到有效回复".into());
        }
        let reply = io.read(port.input_len as usize, Duration::from_millis(150))?;
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
    state.identity = "VGN V98 Pro · 2.4 GHz 接收器".into();
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
            _ => Err(
                "VGN V98 Pro 插件当前提供电量与充电状态；DPI、改键、宏和灯效控制待型号专属适配"
                    .into(),
            ),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("vgn-v98-pro", handler);
