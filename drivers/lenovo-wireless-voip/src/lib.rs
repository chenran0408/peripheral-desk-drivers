use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn read_state() -> Result<State, String> {
    let Some(i) = hid::enumerate()?.into_iter().find(|i| {
        i.vid == 0x17ef
            && i.pid == 0xa07d
            && i.usage_page == 0xff07
            && i.usage == 0x222
            && i.path.to_ascii_lowercase().contains("mi_03")
            && i.feature_len >= 61
            && i.input_len >= 61
    }) else {
        return Ok(State {
            key: "lenovo-wireless-voip".into(),
            error: "未找到 Lenovo 耳机接收器".into(),
            ..State::default()
        });
    };
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    let mut request = [0; 61];
    request[..2].copy_from_slice(&[0x24, 1]);
    hid::Device::open(&i.path)?.send(&request)?;
    let reply = io.read(i.input_len as usize, Duration::from_millis(1000))?;
    if reply.len() < 8 || reply[..2] != [0x27, 1] || reply[2] != 0 || reply[7] > 100 {
        return Err("Lenovo 耳机未返回有效电量".into());
    }
    Ok(State {
        key: "lenovo-wireless-voip".into(),
        present: true,
        fresh: true,
        percent: Some(reply[7]),
        identity: "Lenovo Wireless VoIP · 17EF:A07D".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅提供只读电量；设备不报告充电状态。".into(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "lenovo-wireless-voip" {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持只读电量".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("lenovo-wireless-voip", handler);
