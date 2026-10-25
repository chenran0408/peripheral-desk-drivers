//! AULA F108 Pro receiver battery adapter from Vigil Watt v1.3.0 (a78e973).
//! Independently implemented from documented frames; no source is copied.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "aula-f108-pro";
const VID: u16 = 0x05ac;
const PID: u16 = 0x024f;
const PAGE: u16 = 0xff60;
const USAGE: u16 = 0x0061;
const REPORT_LEN: usize = 33;

fn matches(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && (i.path.to_ascii_lowercase().contains("&mi_03#")
            || i.product.to_ascii_lowercase().contains("f108"))
}

fn parse(report: &[u8]) -> Option<u8> {
    let payload = if report.len() == REPORT_LEN {
        if report[0] != 0 {
            return None;
        }
        &report[1..]
    } else if report.len() == REPORT_LEN - 1 {
        report
    } else {
        return None;
    };
    if payload[0] != 0x20
        || payload[1] != 0x01
        || payload[2] != 0
        || payload[3] == 0
        || payload[3] > 100
        || payload[..payload.len() - 1]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte))
            != payload[31]
    {
        return None;
    }
    Some(payload[3])
}

fn read_state() -> Result<State, String> {
    let ports = hid::enumerate()?;
    let found: Vec<_> = ports.iter().filter(|i| matches(i)).collect();
    if found.len() != 1 {
        return Err("未找到唯一且带 F108 型号标识的 AULA 接收器通道".into());
    }
    let port = found[0];
    let mut query = [0; REPORT_LEN];
    query[1] = 0x20;
    query[2] = 0x01;
    query[32] = 0x21;
    let mut io = WinTransport::open(&port.path)?;
    io.flush()?;
    io.write(&query, Duration::from_millis(500))?;
    let until = Instant::now() + Duration::from_millis(750);
    let percent = loop {
        if Instant::now() >= until {
            return Err("AULA F108 Pro 电量查询未收到有效回复".into());
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
    state.charging = None;
    state.wired = false;
    let product = port.product.to_ascii_lowercase();
    state.identity = if product.contains("ganss") || product.contains("gs3104") {
        "GANSS GS3104T-Pro · 2.4 GHz 接收器".into()
    } else if product.contains("f108") {
        "狼蛛 F108 Pro · 2.4 GHz 接收器".into()
    } else {
        "AULA F108 Pro / GANSS GS3104T-Pro · 2.4 GHz 接收器".into()
    };
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
                "AULA F108 Pro 插件当前提供接收器电量；充电状态未知，控制功能待型号专属适配".into(),
            ),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("aula-f108-pro", handler);
