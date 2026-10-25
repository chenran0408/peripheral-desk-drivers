fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_razer_kraken_v4::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("Kraken V4 插件目前只查询电量和充电状态".into());
        }
        let mut state =
            serde_json::to_value(peripheral_desk_protocol_razer_kraken_v4::read_state()?)
                .map_err(|e| e.to_string())?;
        if let Some(object) = state.as_object_mut() {
            object.insert(
                "battery_raw".into(),
                object
                    .get("percent")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            );
            if object.get("present").and_then(serde_json::Value::as_bool) == Some(true) {
                object.insert(
                    "connection_path".into(),
                    serde_json::json!("2.4 GHz USB 接收器"),
                );
            }
        }
        Ok(state)
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-kraken-v4", handler);
