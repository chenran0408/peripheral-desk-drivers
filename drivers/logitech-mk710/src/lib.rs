// placeholder: implemented by integration worker
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn exchange(io: &mut WinTransport, request: &[u8], response_len: usize) -> Result<Vec<u8>, String> {
    io.flush()?;
    io.write(request, Duration::from_millis(500))?;
    let end = Instant::now() + Duration::from_millis(800);
    while Instant::now() < end {
        let reply = io.read(response_len, Duration::from_millis(100))?;
        if reply.len() >= 5 && reply[0] == request[0] && reply[1..4] == request[1..4] {
            return Ok(reply[4..].to_vec());
        }
    }
    Err("HID++ 1.0 查询超时".into())
}
fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let receiver = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x046d
                && i.pid == 0xc52b
                && i.usage_page == 0xff00
                && i.usage == 2
                && i.input_len == 20
                && i.output_len == 20
        })
        .ok_or("未找到 Unifying 接收器的长报文集合")?;
    let short = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x046d
                && i.pid == 0xc52b
                && i.usage_page == 0xff00
                && i.usage == 1
                && i.input_len == 7
                && i.output_len == 7
        })
        .ok_or("未找到 Unifying 接收器的短报文集合")?;
    let mut identity_io = WinTransport::open(&receiver.path)?;
    let mut battery_io = WinTransport::open(&short.path)?;
    for slot in 1..=6 {
        // Receiver register 0x2B5, subregister 0x40 + slot - 1: paired device name.
        let mut identity_request = [0u8; 20];
        identity_request[..5].copy_from_slice(&[0x11, 0xff, 0x83, 0xb5, 0x40 + slot - 1]);
        let Ok(name_body) = exchange(&mut identity_io, &identity_request, 20) else {
            continue;
        };
        let count = name_body.get(1).copied().unwrap_or(0) as usize;
        let name = String::from_utf8_lossy(name_body.get(2..2 + count.min(14)).unwrap_or(&[]))
            .to_ascii_uppercase();
        if !name.contains("MK710") && !name.contains("M710") {
            continue;
        }
        let mut reading = None;
        for register in [0x0d, 0x07] {
            let request = [0x10, slot, 0x81, register, 0, 0, 0];
            let Ok(reply) = exchange(&mut battery_io, &request, 7) else {
                continue;
            };
            reading = match register {
                0x0d if reply.len() >= 3 && reply[0] <= 100 => {
                    Some((reply[0], matches!(reply[2] & 0xf0, 0x50 | 0x90)))
                }
                0x07 if reply.len() >= 2 => match reply[0] {
                    7 => Some((100, reply[1] & 0x03 != 0)),
                    5 => Some((75, reply[1] & 0x03 != 0)),
                    3 => Some((50, reply[1] & 0x03 != 0)),
                    1 => Some((25, reply[1] & 0x03 != 0)),
                    _ => None,
                },
                _ => None,
            };
            if reading.is_some() {
                break;
            }
        }
        let (percent, charging) = reading.ok_or("MK710 已确认身份，但无有效 HID++ 1.0 电量回复")?;
        return Ok(State {
            key: "logitech-mk710".into(),
            present: true,
            fresh: true,
            percent: Some(percent),
            charging: Some(charging),
            identity: format!("Logitech {name} · 046D:C52B · slot {slot}"),
            settings_error: "旧 HID++ 电量可能只提供粗略档位。".into(),
            updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ..State::default()
        });
    }
    Err("接收器内未发现 MK710/M710 名称；型号不符时不显示".into())
}
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "logitech-mk710" {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持电量读取".into());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("logitech-mk710", handler);
