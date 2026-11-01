//! Feature-report protocol facts: mouse.xyz Config/env-models.json and HaloBattery (MIT).
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "gwolves-hsk-pro-2";

fn parse(reply: &[u8]) -> Option<(u8, bool)> {
    let at = if reply.first() == Some(&0xa1) {
        0
    } else if reply.get(1) == Some(&0xa1) {
        1
    } else {
        return None;
    };
    if reply.get(at + 1..at + 6) != Some(&[0, 2, 2, 0, 0x83]) {
        return None;
    }
    let percent = *reply.get(at + 7)?;
    (percent <= 100).then_some((percent, *reply.get(at + 6)? != 0))
}

fn read_state() -> Result<State, String> {
    let candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| i.vid == 0x33e4 && matches!(i.pid, 0x6817 | 0x6808) && i.feature_len == 65)
        .collect();
    for i in &candidates {
        let device = hid::Device::open(&i.path)?;
        let mut request = vec![0; i.feature_len as usize];
        request[..6].copy_from_slice(&[0, 0, 2, 2, 0, 0x83]);
        device.send(&request)?;
        for _ in 0..15 {
            std::thread::sleep(Duration::from_millis(50));
            if let Ok(reply) = device.receive(0, i.feature_len as usize) {
                if let Some((percent, charging)) = parse(&reply) {
                    return Ok(State {
                        key: KEY.into(),
                        present: true,
                        fresh: true,
                        percent: Some(percent),
                        charging: Some(charging),
                        wired: i.pid == 0x6808,
                        identity: format!("G-Wolves HSK Pro 2.0 · 33E4:{:04X}", i.pid),
                        updated: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        settings_error: "仅支持只读电量与充电状态；未经本项目实机验证。".into(),
                        ..State::default()
                    });
                }
            }
        }
    }
    Ok(State {
        key: KEY.into(),
        present: !candidates.is_empty(),
        error: "HSK Pro 2.0 未取得有效电量回复".into(),
        ..State::default()
    })
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
peripheral_desk_driver_ffi::export_driver!("gwolves-hsk-pro-2", handler);
