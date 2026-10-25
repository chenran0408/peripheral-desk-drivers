//! Exact-model status query and strict TLV battery parser for ML903 NearLink Se.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "thunderobot-ml903-nearlink-se";
const VID: u16 = 0x369b;
const PID: u16 = 0x0234;
const PAGE: u16 = 0xffa0;
const USAGE: u16 = 2;
const LEN: usize = 64;

fn battery(report: &[u8]) -> Option<u8> {
    if report.len() != LEN || report[0] != 0xa1 || report[1] != 0x02 {
        return None;
    }
    let mut at = 1usize;
    let mut value = None;
    while at < report.len() {
        if report[at] == 0 {
            if report[at..].iter().any(|b| *b != 0) {
                return None;
            }
            return value;
        }
        if at + 2 > report.len() {
            return None;
        }
        let tag = u16::from_be_bytes([report[at], report[at + 1]]);
        let size = match tag {
            0x0204 => 2,
            0x0211 => 4,
            0x0213 => 6,
            0x0225 | 0x0226 => 1,
            _ => return None,
        };
        if at + 2 + size > report.len() {
            return None;
        }
        if tag == 0x0225 {
            if value.is_some() || report[at + 2] > 100 {
                return None;
            }
            value = Some(report[at + 2]);
        }
        at += 2 + size;
    }
    value
}

fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == VID
                && i.pid == PID
                && i.usage_page == PAGE
                && i.usage == USAGE
                && i.input_len as usize == LEN
                && i.output_len as usize == LEN
        })
        .collect();
    if interfaces.len() != 1 {
        return Err(if interfaces.is_empty() {
            "未找到 THUNDEROBOT ML903 NearLink Se 369B:0234 接口".into()
        } else {
            "ML903 接口不唯一，未发送查询".into()
        });
    }
    let i = &interfaces[0];
    let mut transport = WinTransport::open(&i.path)?;
    transport.flush()?;
    let mut query = [0; LEN];
    query[0] = 0xa0;
    query[1] = 0x02;
    transport.write(&query, Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(1500);
    let percent = loop {
        if Instant::now() >= deadline {
            return Err("ML903 未收到 0xA0/0x02 查询后的有效应答".into());
        }
        let reply = transport.read(LEN, Duration::from_millis(200))?;
        if let Some(value) = battery(&reply) {
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
    state.identity =
        format!("THUNDEROBOT ML903 NearLink Se · 接收器 369B:0234 · 0225={percent}% · 充电未知");
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
        let req: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if req.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("ML903 插件只提供电量读取".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("thunderobot-ml903-nearlink-se", handler);
