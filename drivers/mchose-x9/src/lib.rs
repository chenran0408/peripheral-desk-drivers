use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn read_state() -> Result<State, String> {
    let Some(i) = hid::enumerate()?.into_iter().find(|i| {
        i.vid == 0x3837
            && i.pid == 0x6045
            && i.usage_page == 0xff90
            && i.usage == 1
            && i.input_len >= 64
            && i.output_len >= 64
    }) else {
        return Ok(State {
            key: "mchose-x9".into(),
            error: "未找到 X9 接收器状态接口".into(),
            ..State::default()
        });
    };
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    let mut query = [0; 64];
    query[..3].copy_from_slice(&[0x55, 0x65, 1]);
    io.write(&query, Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        let r = io.read(i.input_len as usize, Duration::from_millis(100))?;
        if r.len() >= 4 && r[0] == 0x55 && r[1] == 0x65 && r[3] == 2 && r[2] <= 100 {
            return Ok(State {
                key: "mchose-x9".into(),
                present: true,
                fresh: true,
                percent: Some(r[2]),
                identity: "MCHOSE X9 · 3837:6045".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅提供只读电量；接收器不报告充电状态。".into(),
                ..State::default()
            });
        }
    }
    Err("X9 接收器未返回电量报告".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "mchose-x9" {
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
peripheral_desk_driver_ffi::export_driver!("mchose-x9", handler);
