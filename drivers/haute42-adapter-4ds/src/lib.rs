//! Haute42 Adapter 4DS read-only identity and battery plugin.
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_haute42_4ds::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("Haute42 Adapter 4DS 插件只读取经固件确认的电量，不提供写入功能".into());
        }
        let mut state = serde_json::to_value(peripheral_desk_protocol_haute42_4ds::read_state()?)
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
                    serde_json::json!("USB（Haute42 Adapter 4DS）"),
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
peripheral_desk_driver_ffi::export_driver!("haute42-adapter-4ds", handler);
