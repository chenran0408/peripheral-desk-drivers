use peripheral_desk_protocol_ninjutso_sora_v2::read_state;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "ninjutso-sora-v2" {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("Ninjutso Sora V2 插件仅支持只读电量查询".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("ninjutso-sora-v2", handler);
