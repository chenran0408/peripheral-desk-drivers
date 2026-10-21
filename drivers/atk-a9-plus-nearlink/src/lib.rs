use peripheral_desk_protocol_atk_a9_plus_nearlink::DEVICE_KEY;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => {
                serde_json::to_value(peripheral_desk_protocol_atk_a9_plus_nearlink::read_atk()?)
                    .map_err(|e| e.to_string())
            }
            Some("apply-setting") => {
                let action: peripheral_desk::model::Action = serde_json::from_value(
                    request.get("action").cloned().ok_or("缺少 ATK 设置动作")?,
                )
                .map_err(|e| format!("ATK 设置动作无效：{e}"))?;
                peripheral_desk_protocol_atk_a9_plus_nearlink::apply_action(action)?;
                Ok(serde_json::Value::Null)
            }
            _ => Err("ATK A9 Plus Nearlink 预览支持状态读取与 DPI/回报率设置".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("atk-a9-plus-nearlink", handler);
