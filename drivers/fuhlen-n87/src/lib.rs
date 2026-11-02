use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "fuhlen-n87";

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let Some(i) = interfaces
        .iter()
        .find(|i| i.vid == 0x3151 && i.pid == 0x4011 && i.feature_len == 65)
    else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到腹灵 N87 接收器状态接口".into(),
            ..State::default()
        });
    };
    let device = hid::Device::open(&i.path)?;
    let mut request = [0; 65];
    request[1] = 0xf7;
    device.send(&request)?;
    std::thread::sleep(Duration::from_millis(120));
    for _ in 0..3 {
        let reply = device.receive(0, 65)?;
        if reply.len() >= 5 && reply[2] <= 100 && reply[4] == 0 {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(reply[2]),
                identity: "腹灵 N87 · 3151:4011".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅提供只读电量；接收器不报告充电状态。".into(),
                ..State::default()
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err("腹灵 N87 未返回有效电量".into())
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
peripheral_desk_driver_ffi::export_driver!("fuhlen-n87", handler);
