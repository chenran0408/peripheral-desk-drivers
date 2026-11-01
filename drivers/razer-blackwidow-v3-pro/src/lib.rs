// placeholder: implemented by integration worker
mod battery;
fn handler(key: &str, input: &str) -> String {
    handle(
        key,
        input,
        "razer-blackwidow-v3-pro",
        "BlackWidow V3 Pro",
        (0x025c, 2),
        (0x025a, 2, 0x3f),
    )
}
fn handle(
    key: &str,
    input: &str,
    expected: &str,
    name: &str,
    receiver: (u16, u8),
    cable: (u16, u8, u8),
) -> String {
    let result = (|| {
        if key != expected {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持电量读取".into());
        }
        battery::read_state(expected, name, receiver, cable)
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-blackwidow-v3-pro", handler);
