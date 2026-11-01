// placeholder: implemented by integration worker
#[path = "../../razer-blackwidow-v3-pro/src/battery.rs"]
mod battery;
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "razer-blackwidow-v3-mini" {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持电量读取".into());
        }
        battery::read_state(
            key,
            "BlackWidow V3 Mini HyperSpeed",
            (0x0271, 3),
            (0x0258, 3, 0x1f),
        )
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-blackwidow-v3-mini", handler);
