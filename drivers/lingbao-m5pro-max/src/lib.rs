//! Exact receiver battery query for LINGBAO M5PRO Max.
use peripheral_desk::{hid, model::State};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "lingbao-m5pro-max";
const VID: u16 = 0x3151;
const PID: u16 = 0x402d;
const PAGE: u16 = 0xffff;
const USAGE: u16 = 0x0002;
const FEATURE_LEN: usize = 65;

fn read_state() -> Result<State, String> {
    let devices: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == VID
                && i.pid == PID
                && i.usage_page == PAGE
                && i.usage == USAGE
                && i.feature_len as usize >= FEATURE_LEN
        })
        .collect();
    if devices.len() != 1 {
        return Err(if devices.is_empty() {
            "未找到 LINGBAO M5PRO Max 2.4 GHz 接收器 Feature 接口".into()
        } else {
            "检测到多个 LINGBAO M5PRO Max 接收器接口，未查询".into()
        });
    }
    let interface = &devices[0];
    let hid = hid::Device::open(&interface.path)?;
    let mut request = vec![0; interface.feature_len as usize];
    request[1] = 0xf7;
    hid.send(&request)?;
    std::thread::sleep(Duration::from_millis(20));
    let response = hid.receive(0, interface.feature_len as usize)?;
    if response.len() < 6 || response[0] != 0 || !matches!(response[1], 0 | 1) || response[2] != 0 {
        return Err("LINGBAO M5PRO Max Feature 回复头或应答不匹配".into());
    }
    let percent = response[3];
    if !(2..=100).contains(&percent) {
        return Err(format!("LINGBAO M5PRO Max 电量字段无效/休眠：{percent}"));
    }
    let charging = match response[5] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(percent);
    state.charging = charging;
    state.wired = false;
    state.identity =
        format!("LINGBAO M5PRO Max · 2.4 GHz 接收器 3151:402D · F7 原始字段 {percent}%");
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
        let request: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("LINGBAO M5PRO Max 只提供接收器电量与充电状态".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("lingbao-m5pro-max", handler);
