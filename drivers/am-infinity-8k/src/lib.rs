use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "am-infinity-8k";
fn parse_status(r: &[u8]) -> Option<u8> {
    if r.len() < 4 || r[0] != 5 || r[1] != 0 || r[2] != 0 || r[3] == 0 {
        return None;
    }
    Some(r[3].min(100))
}
fn read_state() -> Result<State, String> {
    let candidates: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x3151
                && i.pid == 0x5007
                && i.usage_page == 0xffff
                && i.usage == 2
                && i.feature_len >= 65
        })
        .collect();
    let mut first_error = None;
    for i in &candidates {
        let result = (|| {
            let device = hid::Device::open(&i.path)?;
            for len in [65, 67] {
                if i.feature_len < len {
                    continue;
                }
                let mut poll = vec![0; len as usize];
                poll[1] = 0xf7;
                device.send(&poll)?;
                std::thread::sleep(Duration::from_millis(30));
                if let Ok(reply) = device.receive(5, 65) {
                    if let Some(percent) = parse_status(&reply) {
                        return Ok(Some(percent));
                    }
                }
            }
            Ok(None)
        })();
        match result {
            Ok(Some(percent)) => {
                return Ok(State {
                    key: KEY.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: None,
                    identity: "AM Infinity 8K · 3151:5007 · FFFF:0002".into(),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅支持只读电量；状态报告不含充电标志。".into(),
                    ..State::default()
                })
            }
            Ok(None) => {}
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }
    Ok(State {
        key: KEY.into(),
        present: !candidates.is_empty(),
        error: first_error.unwrap_or_else(|| "AM Infinity 8K 未取得有效电量回复".into()),
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
            return Err("只支持读取电量".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("am-infinity-8k", handler);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status() {
        assert_eq!(parse_status(&[5, 0, 0, 64, 1, 1, 1, 2]), Some(64));
        assert_eq!(parse_status(&[5, 0, 0, 0]), None);
    }
}
