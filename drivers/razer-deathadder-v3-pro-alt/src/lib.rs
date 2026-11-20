//! Razer DeathAdder V3 Pro (alternate receiver) battery adapter based on Voltide v2.1.2.
use peripheral_desk_protocol_razer::pointing::{request, Model};

fn handler(key: &str, input: &str) -> String {
    request(Model::HyperPolling, key, input, read_battery)
}
fn read_battery(key: &str, input: &str) -> String {
    let result = (|| -> Result<serde_json::Value, String> {
        if key != "razer-deathadder-v3-pro-alt" {
            return Err("driver key mismatch".into());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("invalid request JSON: {e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("此插件支持电量读取和板载指针编辑，此项操作未实现。".into());
        }
        let mut state = peripheral_desk_protocol_razer_wireless::read_state(
            "razer-deathadder-v3-pro-alt",
            "DeathAdder V3 Pro (alternate receiver)",
            0x00C3,
        )?;
        // The shared battery-only provider sets a static capability notice.
        state.settings_error.clear();
        serde_json::to_value(state).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok": true, "value": value, "error": ""}),
        Err(error) => serde_json::json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v3-pro-alt", handler);
