use peripheral_desk::{hid, model::State};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "darmoshark-m3";

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let Some(i) = interfaces.iter().find(|i| {
        i.vid == 0x248a && i.pid == 0xff30 && i.usage_page == 0x8c && i.feature_len == 21
    }) else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 Darmoshark M3 接收器状态接口".into(),
            ..State::default()
        });
    };
    let device = hid::Device::open(&i.path)?;
    let mut frame = [0; 21];
    frame[0] = 0x51;
    frame[1] = 3;
    device.send(&frame)?;
    std::thread::sleep(Duration::from_millis(30));
    let bond = device.receive(0x51, 21)?;
    if bond.len() < 8
        || bond[0] != 0x51
        || bond[1] != 3
        || u16::from_le_bytes([bond[3], bond[4]]) != 0x248a
        || u16::from_le_bytes([bond[5], bond[6]]) != 0xff12
        || bond[7] == 0
    {
        return Err("Darmoshark M3 接收器未连接匹配的 M3 鼠标".into());
    }
    frame[1] = 6;
    device.send(&frame)?;
    let until = Instant::now() + Duration::from_millis(350);
    while Instant::now() < until {
        let reply = device.receive(0x51, 21)?;
        if reply.len() >= 12
            && reply[0] == 0x51
            && reply[1] == 6
            && reply[2] == 1
            && reply[11] <= 100
        {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(reply[11]),
                identity: "Darmoshark M3 · 248A:FF30".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅提供只读电量；此状态回复不包含可靠充电标志。".into(),
                ..State::default()
            });
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err("Darmoshark M3 未返回电量回复（可能休眠）".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
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
peripheral_desk_driver_ffi::export_driver!("darmoshark-m3", handler);
