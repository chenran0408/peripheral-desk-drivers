//! Razer DeathAdder V3 Pro (alternate receiver) battery adapter based on Voltide v2.1.2.
fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<serde_json::Value, String> {
        if key != "razer-deathadder-v3-pro-alt" {
            return Err("driver key mismatch".into());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("invalid request JSON: {e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("This plugin provides battery and connection state only; DPI, polling, key mapping, macros and lighting are unavailable.".into());
        }
        Ok(
            serde_json::to_value(peripheral_desk_protocol_razer_wireless::read_state(
                "razer-deathadder-v3-pro-alt",
                "DeathAdder V3 Pro (alternate receiver)",
                0x00C3,
            ))
            .map_err(|e| e.to_string())?,
        )
    })();
    match result {
        Ok(value) => serde_json::json!({"ok": true, "value": value, "error": ""}),
        Err(error) => serde_json::json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v3-pro-alt", handler);
