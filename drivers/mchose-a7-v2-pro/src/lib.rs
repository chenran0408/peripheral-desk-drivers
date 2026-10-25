//! Battery and charging status for the contributor-verified A7 V2 Pro
//! receiver's passive MCHOSE mouse V2 E2 status reports.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "mchose-a7-v2-pro";
const VID: u16 = 0x3837;
const PID: u16 = 0x100a;
const REPORT_ID: u8 = 0x13;
const MIN_REPORT_LEN: usize = 60;
const MAX_REPORT_LEN: usize = 4096;
const POKE_REPORT_ID: u8 = 0x11;

fn matches(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page >= 0xff00
        && i.input_len as usize >= MIN_REPORT_LEN
        && i.output_len as usize >= MIN_REPORT_LEN
}

fn parse_status(input: &[u8]) -> Option<(u8, bool)> {
    for offset in [1usize, 0] {
        if input.len() < offset + 5 {
            continue;
        }
        // Windows includes report ID 0x13 unchanged at [0]. Payload bytes are
        // inverted by the receiver; tolerate the upstream-documented input
        // shape without a report ID as well.
        if offset == 1 && input[0] != REPORT_ID {
            continue;
        }
        let first = input[offset] ^ 0xff;
        if first != 0xe2 {
            continue;
        }
        let charging = input[offset + 3] ^ 0xff == 1;
        let percent = input[offset + 4] ^ 0xff;
        if (2..=100).contains(&percent) {
            return Some((percent, charging));
        }
    }
    None
}

fn listen(
    transport: &mut WinTransport,
    deadline: Instant,
    report_len: usize,
) -> Result<Option<(u8, bool)>, String> {
    while Instant::now() < deadline {
        let input = transport.read(report_len, Duration::from_millis(300))?;
        if let Some(status) = parse_status(&input) {
            return Ok(Some(status));
        }
    }
    Ok(None)
}

fn poke(report_len: usize) -> Vec<u8> {
    let mut frame = vec![0xff; report_len];
    frame[0] = POKE_REPORT_ID;
    frame[1] = 0xf9; // 0x06 inverted; report ID is not inverted.
    frame
}

fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    if interfaces.len() != 1 {
        return Err(if interfaces.is_empty() {
            "未找到 MCHOSE A7 V2 Pro 专用 3837:100A 鼠标状态通道".into()
        } else {
            "检测到多个 MCHOSE A7 V2 Pro 状态通道，未发送查询".into()
        });
    }
    let interface = &interfaces[0];
    let in_len = interface.input_len as usize;
    let out_len = interface.output_len as usize;
    if in_len > MAX_REPORT_LEN || out_len > MAX_REPORT_LEN {
        return Err("MCHOSE A7 V2 Pro HID 报告长度超出支持范围".into());
    }
    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;

    // Match Vigil Watt's flow: prefer the naturally pushed passive E2 status;
    // only if none arrives, send the vendor's documented status-wake frame.
    let status = listen(
        &mut transport,
        Instant::now() + Duration::from_secs(3),
        in_len,
    )?;
    let status = if let Some(status) = status {
        status
    } else {
        transport.write(&poke(out_len), Duration::from_millis(500))?;
        listen(
            &mut transport,
            Instant::now() + Duration::from_secs(3),
            in_len,
        )?
        .ok_or("MCHOSE A7 V2 Pro 未收到有效 E2 电量状态")?
    };

    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(status.0);
    state.charging = Some(status.1);
    state.wired = false;
    state.identity = format!("MCHOSE A7 V2 Pro · 2.4 GHz 接收器 3837:100A · 被动 E2 状态");
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
        match request.get("op").and_then(Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("MCHOSE A7 V2 Pro 插件仅提供电量与充电状态；控制功能尚未接入".into()),
        }
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("mchose-a7-v2-pro", handler);
