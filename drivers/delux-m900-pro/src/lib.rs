//! Passive zero-write receiver battery listener for DELUX M900 Pro.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "delux-m900-pro";
const VID: u16 = 0x1d57;
const PID: u16 = 0xfa65;
const PAGE: u16 = 0x000a;
const USAGE: u16 = 0x0000;
const REPORT_LEN: usize = 5;
const REPORT_ID: u8 = 0x03;

fn read_state() -> Result<State, String> {
    let candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == VID
                && i.pid == PID
                && i.usage_page == PAGE
                && i.usage == USAGE
                && i.input_len as usize == REPORT_LEN
                && i.output_len == 0
                && i.feature_len == 0
        })
        .collect();
    if candidates.len() != 1 {
        return Err(if candidates.is_empty() {
            "未找到 DELUX M900 Pro 专用 000A:0000 被动接口".into()
        } else {
            "DELUX M900 Pro 被动状态接口不唯一".into()
        });
    }
    let i = &candidates[0];
    let mut transport = WinTransport::open(&i.path)?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let report = transport.read(REPORT_LEN, Duration::from_millis(400))?;
        if report.len() != REPORT_LEN
            || report[0] != REPORT_ID
            || report[1] != 0x30
            || report[2] != 0x40
        {
            continue;
        }
        let flag = report[3];
        let percent = report[4];
        if percent > 100 {
            continue;
        }
        let charging = match flag {
            0x01 => Some(false),
            0x02 => None,
            0x03 => Some(true),
            _ => None,
        };
        let mut state = State::default();
        state.key = KEY.into();
        state.present = true;
        state.fresh = true;
        state.percent = Some(percent);
        state.charging = charging;
        state.wired = false;
        state.identity = format!(
            "DELUX M900 Pro · 2.4 GHz 接收器 1D57:FA65 · 000A:0000 · 原始标志 0x{flag:02X}"
        );
        state.updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        return Ok(state);
    }
    Err("3 秒内未收到 DELUX M900 Pro 30 40 状态帧".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("DELUX M900 Pro 插件仅被动读取电量状态".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("delux-m900-pro", handler);
