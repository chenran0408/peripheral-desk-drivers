const MODEL: peripheral_desk_protocol_glorious_mow::Model =
    peripheral_desk_protocol_glorious_mow::Model {
        key: "glorious-model-d-minus-wireless",
        name: "Glorious Model D- Wireless",
        wired: 0x2014,
        receiver: 0x2025,
    };
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != MODEL.key {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持只读电量".into());
        }
        serde_json::to_value(peripheral_desk_protocol_glorious_mow::read_state(&MODEL)?)
            .map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("glorious-model-d-minus-wireless", handler);
