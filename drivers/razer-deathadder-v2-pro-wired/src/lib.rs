fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_razer_deathadder_v2_pro::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(
                peripheral_desk_protocol_razer_deathadder_v2_pro::read_state()?,
            )
            .map_err(|e| e.to_string()),
            Some("lighting-read") => {
                let config_dir = request
                    .get("config_dir")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "缺少 config_dir".to_string())?;
                peripheral_desk_protocol_razer_deathadder_v2_pro::lighting_read(config_dir)
            }
            Some("lighting-apply") => {
                let config_dir = request
                    .get("config_dir")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "缺少 config_dir".to_string())?;
                let config = request
                    .get("config")
                    .ok_or_else(|| "缺少 config".to_string())?;
                peripheral_desk_protocol_razer_deathadder_v2_pro::lighting_apply(config_dir, config)
            }
            Some("lighting-switch") => {
                let config_dir = request
                    .get("config_dir")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "缺少 config_dir".to_string())?;
                let enabled = request
                    .get("enabled")
                    .and_then(serde_json::Value::as_bool)
                    .ok_or_else(|| "缺少 enabled".to_string())?;
                peripheral_desk_protocol_razer_deathadder_v2_pro::lighting_switch(
                    config_dir, enabled,
                )
            }
            Some("apply-setting") => {
                let action = request
                    .get("action")
                    .ok_or_else(|| "缺少驱动参数：action".to_string())?;
                peripheral_desk_protocol_razer_deathadder_v2_pro::apply_setting_value(action)?;
                Ok(serde_json::Value::Null)
            }
            Some(operation) => Err(format!("DeathAdder V2 Pro 不支持操作：{operation}")),
            None => Err("缺少驱动操作类型".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v2-pro-wired", handler);
