//! Read-only battery reader for the exact AULA L9 Ultra receiver endpoint.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::json;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "aula-l9-ultra";
const VID: u16 = 0x38a6;
const PID: u16 = 0x28fd;
const PAGE: u16 = 0xffa8;
const OUT_ID: u8 = 0x0b;
const IN_ID: u8 = 0x0a;
const COMMAND: u8 = 0x81;
const CHARGING_INDEX: usize = 17;
const LEVEL_INDEX: usize = 18;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == PAGE
        && i.output_len >= 3
        && i.input_len as usize > LEVEL_INDEX
}

fn query() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?.into_iter().filter(supported).collect();
    if interfaces.len() != 1 {
        return Err(if interfaces.is_empty() {
            "未找到 AULA L9 Ultra 的 FFA8 状态接口".into()
        } else {
            "检测到多个 AULA L9 Ultra 状态接口，未发送查询".into()
        });
    }
    let interface = &interfaces[0];
    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;
    let mut request = vec![0; interface.output_len as usize];
    request[0] = OUT_ID;
    request[1] = COMMAND;
    transport.write(&request, Duration::from_millis(500))?;

    let until = Instant::now() + Duration::from_millis(1200);
    let (percent, charging) = loop {
        if Instant::now() >= until {
            return Err("AULA L9 Ultra 未收到匹配的 0x81 状态回复".into());
        }
        let reply = transport.read(interface.input_len as usize, Duration::from_millis(100))?;
        if reply.len() <= LEVEL_INDEX || reply[0] != IN_ID || reply[1] != COMMAND {
            continue;
        }
        let level = reply[LEVEL_INDEX];
        if level > 100 {
            return Err(format!("AULA L9 Ultra 电量字段超出 0–100：{level}"));
        }
        let charging = match reply[CHARGING_INDEX] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        break (level, charging);
    };
    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(percent);
    state.charging = charging;
    state.wired = false;
    state.identity = "AULA L9 Ultra · 2.4 GHz 接收器 38A6:28FD · FFA8".into();
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
        let req: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if req.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("AULA L9 Ultra 插件只提供电量和充电状态读取".into());
        }
        serde_json::to_value(query()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("aula-l9-ultra", handler);
