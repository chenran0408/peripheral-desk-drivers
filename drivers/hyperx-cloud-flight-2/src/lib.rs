fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_hyperx_cloud_flight_2::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("Cloud Flight 2 插件仅被动读取电量与充电状态，不包含控制写入".into());
        }
        let mut state =
            serde_json::to_value(peripheral_desk_protocol_hyperx_cloud_flight_2::read_state()?)
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
                    serde_json::json!("USB 接收器被动状态流"),
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
peripheral_desk_driver_ffi::export_driver!("hyperx-cloud-flight-2", handler);
