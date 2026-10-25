//! Battery-only support for the first-generation MCHOSE G87 receiver.
//!
//! The device is deliberately restricted to 41E4:2001's 20-byte FF02
//! MI_01&Col01 configuration channel. The QHW protocol used by newer 3837
//! receivers is a different protocol and is not accepted here.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "mchose-g87-v1";
const VID: u16 = 0x41e4;
const PID: u16 = 0x2001;
const PAGE: u16 = 0xff02;
const REPORT_ID: u8 = 0x13;
const REPORT_LEN: usize = 20;
const INTERFACE_MARKER: &str = "&mi_01&col01#";

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

fn frame(command: u8, parameter: u8) -> [u8; REPORT_LEN] {
    let mut packet = [0; REPORT_LEN];
    packet[0] = REPORT_ID;
    packet[1] = command;
    packet[2] = parameter;
    packet[REPORT_LEN - 1] = checksum(&packet[..REPORT_LEN - 1]);
    packet
}

fn matches(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == PAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains(INTERFACE_MARKER)
}

fn valid_checksum(reply: &[u8]) -> bool {
    reply.len() == REPORT_LEN && checksum(&reply[..REPORT_LEN - 1]) == reply[REPORT_LEN - 1]
}

fn wait_for(
    transport: &mut WinTransport,
    until: Instant,
    matches: impl Fn(&[u8]) -> bool,
) -> Result<Vec<u8>, String> {
    while Instant::now() < until {
        let reply = transport.read(REPORT_LEN, Duration::from_millis(100))?;
        if matches(&reply) {
            return Ok(reply);
        }
    }
    Err("MCHOSE G87 V1 未收到匹配的 HID 回复".into())
}

fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    if interfaces.len() != 1 {
        return Err(if interfaces.is_empty() {
            "未找到 MCHOSE G87 V1 专用 41E4:2001 电量通道".into()
        } else {
            "检测到多个 MCHOSE G87 V1 电量通道，未发送查询".into()
        });
    }
    let interface = &interfaces[0];
    let mut transport = WinTransport::open(&interface.path)?;
    transport.flush()?;

    // 41E4:2001 stays silent until the V1 dongle session is opened.
    let open_session = frame(0x05, 0x01);
    transport.write(&open_session, Duration::from_millis(500))?;
    let ack = wait_for(
        &mut transport,
        Instant::now() + Duration::from_millis(1200),
        |r| {
            valid_checksum(r)
                && r[0] == REPORT_ID
                && r[1] == 0x05
                && r[2] == 0x01
                && r[3] == 0x00
                && r[4] == 0x0a
        },
    )?;
    if ack[5] != 0x03 {
        return Err("MCHOSE G87 V1 会话应答格式不匹配".into());
    }

    let battery_request = frame(0x4a, 0x01);
    transport.write(&battery_request, Duration::from_millis(500))?;
    let reply = wait_for(
        &mut transport,
        Instant::now() + Duration::from_millis(1200),
        |r| {
            valid_checksum(r)
                && r[0] == REPORT_ID
                && r[1] == 0x4a
                && r[2] == 0x01
                && r[3] == 0x00
                && r[4] == 0x02
                && (2..=100).contains(&r[5])
        },
    )?;

    let charge_flag = reply[6];
    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(reply[5]);
    // Only bit 0x10 is confirmed as charging. 0x01 is discharging; other
    // values remain unknown rather than being guessed as a charging state.
    state.charging = if charge_flag & 0x10 != 0 {
        Some(true)
    } else if charge_flag == 0x01 {
        Some(false)
    } else {
        None
    };
    state.wired = false;
    state.identity =
        format!("MCHOSE G87 V1 · 2.4 GHz 接收器 41E4:2001 · 原始充电标志 0x{charge_flag:02X}");
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
            _ => Err("MCHOSE G87 V1 插件当前提供电量与已确认的充电状态；控制功能尚未接入".into()),
        }
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("mchose-g87-v1", handler);
