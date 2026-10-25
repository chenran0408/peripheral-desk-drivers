//! Read-only Feature-mailbox battery support for the EWEADN S9 family.
use peripheral_desk::{hid, model::State};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "eweadn-s9-series";
const VID: u16 = 0x372e;
const PID: u16 = 0x1026;
const PAGE: u16 = 0xff00;
const USAGE: u16 = 0x0001;
const REPORT_ID: u8 = 0x03;
const FEATURE_LEN: usize = 64;

fn query() -> Result<State, String> {
    let matches: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == VID
                && i.pid == PID
                && i.usage_page == PAGE
                && i.usage == USAGE
                && i.feature_len as usize == FEATURE_LEN
        })
        .collect();
    if matches.len() != 1 {
        return Err(if matches.is_empty() {
            "未找到 EWEADN S9 系列 FF00:0001 Feature 接口".into()
        } else {
            "检测到多个 EWEADN S9 Feature 接口，未发送查询".into()
        });
    }
    let interface = &matches[0];
    let device = hid::Device::open(&interface.path)?;
    // Vigil Watt frame payload is 63 bytes; Windows HID includes report ID in
    // the 64-byte Feature buffer passed to HidD_SetFeature/GetFeature.
    let mut request = [0; FEATURE_LEN];
    request[0] = REPORT_ID;
    request[1] = 0x22; // checksum of payload bytes [1..63]
    request[2] = 0x50;
    request[3] = 0x00;
    request[4] = 0x02;
    request[5] = 0x4f;
    request[6] = 0x81;
    let checksum = request[2..].iter().fold(0u8, |sum, b| sum.wrapping_add(*b));
    request[1] = checksum;
    device.send(&request)?;
    std::thread::sleep(Duration::from_millis(20));
    let reply = device.receive(REPORT_ID, FEATURE_LEN)?;
    if reply.len() < 7 || reply[0] != REPORT_ID || reply[1] != 0x50 {
        return Err("EWEADN S9 未收到匹配的 0x50 Feature 回复".into());
    }
    let raw = reply[6];
    let percent = raw & 0x7f;
    if percent == 0 || percent > 100 {
        return Err(format!("EWEADN S9 返回无效或休眠态电量字段 0x{raw:02X}"));
    }
    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(percent);
    state.charging = Some(raw & 0x80 != 0);
    state.wired = false;
    state.identity =
        format!("EWEADN S9 系列 · 2.4 GHz 接收器 372E:1026 · Feature 03 · 原始电量 0x{raw:02X}");
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
            return Err("EWEADN S9 插件只提供电量与充电状态读取".into());
        }
        serde_json::to_value(query()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("eweadn-s9-series", handler);
