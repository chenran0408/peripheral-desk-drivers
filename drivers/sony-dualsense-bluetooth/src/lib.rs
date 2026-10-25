use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "sony-dualsense-bluetooth";
const REPORT_LEN: usize = 78;
const BLUETOOTH_HID_UUID: &str = "{00001124-0000-1000-8000-00805f9b34fb}";

fn supported(i: &hid::Interface) -> bool {
    i.vid == 0x054c
        && i.pid == 0x0ce6
        && i.usage_page == 1
        && i.usage == 5
        && i.input_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains(BLUETOOTH_HID_UUID)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn parse(frame: &[u8]) -> Option<(u8, Option<bool>)> {
    if frame.len() != REPORT_LEN || frame[0] != 0x31 {
        return None;
    }
    let mut checksum_input = [0u8; 75];
    checksum_input[0] = 0xa1;
    checksum_input[1..].copy_from_slice(&frame[..74]);
    let expected = crc32(&checksum_input);
    let actual = u32::from_le_bytes(frame[74..78].try_into().ok()?);
    if actual != expected {
        return None;
    }
    let status = frame[54];
    let level = status & 0x0f;
    let percent = match level {
        0..=10 => (level * 10 + 5).min(100),
        _ => return None,
    };
    let charging = match status >> 4 {
        0 => Some(false),
        1 => Some(true),
        2 => None,
        _ => return None,
    };
    Some((percent, charging))
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 DualSense 蓝牙电量通道".into(),
            settings_error: "只监听物理蓝牙 DualSense；USB、有线、Edge 和 4DS 适配器不适用。"
                .into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 DualSense 蓝牙输入接口，停止监听".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    let deadline = std::time::Instant::now() + Duration::from_millis(1200);
    while std::time::Instant::now() < deadline {
        let frame = io.read(REPORT_LEN, Duration::from_millis(250))?;
        if let Some((percent, charging)) = parse(&frame) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging,
                wired: false,
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                identity: "Sony DualSense · Bluetooth · 054C:0CE6 · 10% 档位估算".into(),
                settings_error:
                    "电量以手柄约 10% 档位的中点估算；此插件被动读取，不含按键、宏或灯光写入。"
                        .into(),
                ..State::default()
            });
        }
    }
    Err("DualSense 蓝牙接口未收到 CRC 正确的 0x31 电量帧".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件只被动读取 DualSense 蓝牙电量，不发送设备写入".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("sony-dualsense-bluetooth", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_crc_status_and_charge_codes() {
        let mut frame = [0u8; REPORT_LEN];
        frame[0] = 0x31;
        frame[54] = 0x01;
        let mut input = [0u8; 75];
        input[0] = 0xa1;
        input[1..].copy_from_slice(&frame[..74]);
        frame[74..].copy_from_slice(&crc32(&input).to_le_bytes());
        assert_eq!(parse(&frame), Some((15, Some(false))));
        frame[54] = 0x21;
        input[1..].copy_from_slice(&frame[..74]);
        frame[74..].copy_from_slice(&crc32(&input).to_le_bytes());
        assert_eq!(parse(&frame), Some((15, None)));
        frame[10] ^= 1;
        assert_eq!(parse(&frame), None);
    }
}
