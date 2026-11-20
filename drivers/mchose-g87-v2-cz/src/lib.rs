//! G87 V2 on the shared 3837:3033 CZ receiver, claimed by its owner.
use peripheral_desk::{hid, model::State};
use peripheral_desk_protocol_mchose_cz as cz;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const KEY: &str = "mchose-g87-v2-cz";
fn matches(i: &hid::Interface) -> bool {
    i.vid == 0x3837
        && i.pid == 0x3033
        && i.usage_page == 0xff31
        && i.usage == 0x74
        && i.input_len as usize == cz::LEN
        && i.output_len as usize == cz::LEN
        && i.path.to_ascii_lowercase().contains("&mi_02")
}

fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    let [interface] = interfaces.as_slice() else {
        return Err("请连接一个已认领为 G87 V2 的 3837:3033 接收器".into());
    };
    let battery = cz::read_battery(&interface.path)?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(battery.percent),
        charging: Some(battery.charging),
        battery_raw: Some(battery.raw),
        connection_path: Some("2.4 GHz · 3837:3033 · MI_02 · FF31:0074".into()),
        identity: "G87 V2（用户已确认型号）".into(),
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
        let request: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("此接收器插件支持读取电量和充电状态".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => json!({"ok": true, "value": value, "error": ""}),
        Err(error) => json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("mchose-g87-v2-cz", handler);
