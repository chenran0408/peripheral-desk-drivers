//! Read-only AK870 receiver battery and online status.
use peripheral_desk::{hid, model::State};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "ajazz-ak870";
const VID: u16 = 0x3151;
const PID: u16 = 0x4011;
const PAGE: u16 = 0xffff;
const USAGE: u16 = 2;
const FEATURE_LEN: usize = 65;

fn read_state() -> Result<State, String> {
    let found: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == VID
                && i.pid == PID
                && i.usage_page == PAGE
                && i.usage == USAGE
                && i.feature_len as usize == FEATURE_LEN
        })
        .collect();
    if found.len() != 1 {
        return Err(if found.is_empty() {
            "未找到 AJAZZ AK870 3151:4011 Feature 接口".into()
        } else {
            "检测到多个 AJAZZ AK870 Feature 接口，未查询".into()
        });
    }
    let i = &found[0];
    let hid = hid::Device::open(&i.path)?;
    let mut query = vec![0; FEATURE_LEN];
    query[1] = 0xf7;
    query[8] = 0x08;
    hid.send(&query)?;
    std::thread::sleep(Duration::from_millis(100));
    let reply = hid.receive(0, FEATURE_LEN)?;
    if !matches!(reply.len(), 64 | 65)
        || reply[0] != 0
        || reply[1] > 1
        || reply[2] > 100
        || reply[4] > 1
        || reply[5] > 1
        || reply[6] > 1
        || !matches!(reply[7], 1 | 3)
    {
        return Err("AJAZZ AK870 Feature 应答字段不匹配或电量越界".into());
    }
    let online = reply[4] == 0;
    let mut state = State::default();
    state.key = KEY.into();
    state.present = online;
    state.fresh = true;
    state.percent = Some(reply[2]);
    state.charging = None;
    state.wired = false;
    state.identity = format!("AJAZZ AK870 · 接收器 3151:4011 · 原始在线标志 {}", reply[4]);
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
            return Err("AJAZZ AK870 插件仅提供电量和连接状态".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("ajazz-ak870", handler);
