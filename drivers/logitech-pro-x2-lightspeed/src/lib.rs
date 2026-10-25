//! Logitech PRO X 2 LIGHTSPEED receiver battery plugin.
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_logitech_pro_x2_lightspeed::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("PRO X 2 插件只提供只读电量查询；音效、均衡器及其他写入未开放".into());
        }
        let mut state = serde_json::to_value(
            peripheral_desk_protocol_logitech_pro_x2_lightspeed::read_state()?,
        )
        .map_err(|e| e.to_string())?;
        if let Some(object) = state.as_object_mut() {
            let raw = object
                .get("percent")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            object.insert("battery_raw".into(), raw);
            if object.get("present").and_then(serde_json::Value::as_bool) == Some(true) {
                object.insert(
                    "connection_path".into(),
                    serde_json::json!("Logitech LIGHTSPEED 接收器"),
                );
            }
        }
        Ok(state)
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("logitech-pro-x2-lightspeed", handler);
