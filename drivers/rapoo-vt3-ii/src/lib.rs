//! Passive, zero-write battery monitor for the Rapoo VT3 II receiver.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "rapoo-vt3-ii";
const VID: u16 = 0x24ae;
const PID: u16 = 0x1412;
const PAGE: u16 = 0xff00;
const REPORT_LEN: usize = 19;
const REPORT_ID: u8 = 0x07;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && (i.usage_page == PAGE || i.usage == PAGE)
        && i.input_len as usize == REPORT_LEN
}

fn read_state() -> Result<State, String> {
    let candidates: Vec<_> = hid::enumerate()?.into_iter().filter(supported).collect();
    if candidates.len() != 1 {
        return Err(if candidates.is_empty() {
            "未找到 Rapoo VT3 II 24AE:1412 的 19 字节状态接口".into()
        } else {
            "Rapoo VT3 II 状态接口不唯一，未读取".into()
        });
    }
    let interface = &candidates[0];
    let mut transport = WinTransport::open(&interface.path)?;
    // This receiver is passive; waiting over one documented heartbeat period
    // does not send any report to the mouse.
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let report = transport.read(REPORT_LEN, Duration::from_millis(400))?;
        if report.len() != REPORT_LEN || report[0] != REPORT_ID {
            continue;
        }
        let percent = report[8];
        if !(1..=100).contains(&percent) {
            continue;
        }
        let charging = match report[7] {
            1 => Some(false),
            2 => Some(true),
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
            "Rapoo VT3 二代 · 2.4 GHz 接收器 24AE:1412 · report 07 · 原始充电标志 0x{:02X}",
            report[7]
        );
        state.updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        return Ok(state);
    }
    Err("5 秒内未收到 Rapoo VT3 II 新鲜 report 07；鼠标静止/休眠时接收器会静音".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("Rapoo VT3 II 插件是纯被动电量读取".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("rapoo-vt3-ii", handler);
