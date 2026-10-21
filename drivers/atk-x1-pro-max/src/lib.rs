use peripheral_desk_protocol_atk::DEVICE_KEY;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("ATK X1 Pro Max 预览仅提供只读电量与充电状态；没有设置写入操作".into());
        }
        serde_json::to_value(peripheral_desk_protocol_atk::read_atk()?)
            .map_err(|error| error.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("atk-x1-pro-max", handler);
