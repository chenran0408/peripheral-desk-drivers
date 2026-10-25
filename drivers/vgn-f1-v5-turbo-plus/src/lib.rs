//! VGN F1 V5 Turbo+ battery/charging reader.
//! Protocol scope and upstream evidence are recorded in `PROVENANCE.md`.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "vgn-f1-v5-turbo-plus";
const VID: u16 = 0x391d;
const WIRELESS_PID: u16 = 0x1a23;
const WIRED_PID: u16 = 0x1026;
const PAGE: u16 = 0xffc1;
const USAGE: u16 = 0x0001;
const REPORT_LEN: usize = 64;

fn matches(i: &Interface) -> bool {
    i.vid == VID
        && matches!(i.pid, WIRELESS_PID | WIRED_PID)
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
}

fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    let interface = match interfaces.as_slice() {
        [one] => one,
        [] => {
            return Err(
                "未找到 VGN F1 V5 Turbo+ 的 64 字节 FFC1:0001 接口（391D:1A23/1026）".into(),
            )
        }
        _ => return Err("检测到多个 VGN F1 V5 Turbo+ 配置接口；已停止读取".into()),
    };
    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;
    let mut request = [0; REPORT_LEN];
    request[0] = 0xb3;
    request[1] = 0x06;
    transport.write(&request, Duration::from_millis(300))?;
    let reply = transport.read(REPORT_LEN, Duration::from_millis(700))?;
    if reply.len() != REPORT_LEN {
        return Err(format!(
            "VGN 配置回复长度不匹配：期望 {REPORT_LEN}，收到 {}",
            reply.len()
        ));
    }
    if reply[0] != 0xb4 || reply[1] != 0x06 {
        return Err(format!(
            "VGN 回复不是 F1 V5 配置帧：report=0x{:02X}, command=0x{:02X}",
            reply[0], reply[1]
        ));
    }
    let raw = reply[20];
    let percent = raw & 0x7f;
    if percent > 100 {
        return Err(format!("F1 V5 电量字段超出 0–100：{percent}"));
    }
    let wired = interface.pid == WIRED_PID;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        wired,
        percent: Some(percent),
        charging: Some(raw & 0x80 != 0),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: format!("VGN F1 V5 Turbo+ · 391D:{:04X} · FFC1:0001", interface.pid),
        settings_error: "本插件仅读取电量与充电状态；DPI、回报率、灯效、改键和宏尚未适配".into(),
        error: String::new(),
        ..Default::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if key != KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let request: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match request.get("op").and_then(Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("VGN F1 V5 Turbo+ 插件只支持电量和充电状态读取".into()),
        }
    })();
    match result {
        Ok(value) => json!({"ok": true, "value": value, "error": ""}).to_string(),
        Err(error) => json!({"ok": false, "value": null, "error": error}).to_string(),
    }
}

peripheral_desk_driver_ffi::export_driver!("vgn-f1-v5-turbo-plus", handler);
