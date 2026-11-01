fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_hyperx_cloud::III_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let req: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if req.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持只读电量与充电状态".into());
        }
        serde_json::to_value(peripheral_desk_protocol_hyperx_cloud::read_state(
            peripheral_desk_protocol_hyperx_cloud::Model::Iii,
        )?)
        .map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("hyperx-cloud-iii-wireless", handler);
