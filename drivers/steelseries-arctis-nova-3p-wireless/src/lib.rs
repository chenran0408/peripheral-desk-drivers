use peripheral_desk_protocol_steelseries_arctis as protocol;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != protocol::NOVA3P_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持只读电量".into());
        }
        serde_json::to_value(protocol::read_state_for(
            protocol::NOVA3P_KEY,
            protocol::NOVA3P,
            false,
        )?)
        .map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("steelseries-arctis-nova-3p-wireless", handler);
