//! Razer Naga V2 HyperSpeed battery adapter based on Voltide v2.1.2.
fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<serde_json::Value, String> {
        if key != "razer-naga-v2-hyperspeed" {
            return Err("driver key mismatch".into());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("invalid request JSON: {e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("This plugin provides battery and connection state only; DPI, polling, key mapping, macros and lighting are unavailable.".into());
        }
        serde_json::to_value(peripheral_desk_protocol_razer_wireless::read_state(
            "razer-naga-v2-hyperspeed",
            "Naga V2 HyperSpeed",
            0x00B4,
        )?)
        .map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok": true, "value": value, "error": ""}),
        Err(error) => serde_json::json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-naga-v2-hyperspeed", handler);
