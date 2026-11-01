//! Battery query for a manually confirmed Azoth Extreme on the Omni receiver.
//! The shared receiver cannot independently identify the keyboard model.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "asus-rog-azoth-extreme";
const VID: u16 = 0x0b05;
const PID: u16 = 0x1ace;
const REPORT: usize = 64;
const KEYBOARD_PATH: &str = "&mi_02&col02#";

fn receiver_key(path: &str) -> Option<String> {
    let parts: Vec<_> = path.split('#').collect();
    if parts.len() < 3 {
        return None;
    }
    let collection = parts[1].to_ascii_lowercase().find("&col")?;
    let instance = parts[2].rfind('&')?;
    Some(format!(
        "{}#{}",
        &parts[1][..collection],
        &parts[2][..instance]
    ))
}

fn matches_keyboard(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0xff00
        && i.usage == 1
        && i.input_len as usize == REPORT
        && i.output_len as usize == REPORT
        && i.path.to_ascii_lowercase().contains(KEYBOARD_PATH)
}

fn make_frame(report_id: u8, command: u8, subcommand: u8) -> [u8; REPORT] {
    let mut frame = [0; REPORT];
    frame[0] = report_id;
    frame[1] = command;
    frame[2] = subcommand;
    frame
}

fn has_paired_keyboard(reply: &[u8]) -> bool {
    if reply.len() != REPORT
        || reply[0] != 1
        || reply[1] != 0xa0
        || reply[2] != 0
        || reply[4] != 0
        || reply[3] as usize > (REPORT - 5) / 4
    {
        return false;
    }
    let entries = reply[5..5 + reply[3] as usize * 4].as_chunks::<4>().0;
    entries
        .iter()
        .filter(|entry| entry[2] == 2 && entry[3] == 4)
        .count()
        == 1
}

fn query(
    path: &str,
    request: &[u8; REPORT],
    valid: impl Fn(&[u8]) -> bool,
) -> Result<Vec<u8>, String> {
    let mut transport = WinTransport::open(path)?;
    transport.flush()?;
    transport.write(request, Duration::from_millis(500))?;
    let until = Instant::now() + Duration::from_millis(700);
    while Instant::now() < until {
        let bytes = transport.read(REPORT, Duration::from_millis(100))?;
        if valid(&bytes) {
            return Ok(bytes);
        }
    }
    Err("ROG Omni 查询超时或未收到匹配回复".into())
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let keyboard = interfaces
        .iter()
        .find(|i| matches_keyboard(i))
        .ok_or("未找到 ROG Omni 键盘通道")?;
    let key = receiver_key(&keyboard.path).ok_or("无法识别 Omni 接收器接口组")?;
    let paired: Vec<_> = interfaces
        .iter()
        .filter(|i| {
            i.vid == VID
                && i.pid == PID
                && (0xff00..=0xff02).contains(&i.usage_page)
                && i.usage == 1
                && i.input_len as usize == REPORT
                && i.output_len as usize == REPORT
                && receiver_key(&i.path).as_deref() == Some(key.as_str())
                && !i.path.to_ascii_lowercase().contains(KEYBOARD_PATH)
        })
        .collect();
    if paired.len() != 1 {
        return Err("ROG Omni 同组控制通道不唯一，未查询配对设备".into());
    }
    query(
        &paired[0].path,
        &make_frame(1, 0xa0, 0),
        has_paired_keyboard,
    )?;

    let reply = query(&keyboard.path, &make_frame(2, 0x12, 1), |r| {
        r.len() == REPORT && r[0] == 2 && r[1] == 0x12 && r[2] == 1 && r[6] <= 100 && r[9] <= 1
    })?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(reply[6]),
        charging: Some(reply[9] == 1),
        identity: "ROG Azoth Extreme（用户手动确认型号）".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("ROG Azoth Extreme 插件仅提供电量与充电状态读取".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("asus-rog-azoth-extreme", handler);
