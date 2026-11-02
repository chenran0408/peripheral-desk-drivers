const KEY: &str = "razer-huntsman-v2-tkl";
const PID: u16 = 0x026b;
const TID: u8 = 0x1f;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| error.to_string())?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("lighting-read") => {
                peripheral_desk_protocol_razer_keyboard_light::lighting_read(KEY, PID, TID)
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
                peripheral_desk_protocol_razer_keyboard_light::lighting_switch(
                    KEY, PID, TID, config_dir, enabled,
                )
            }
            Some("apply-setting") => {
                let action = request
                    .get("action")
                    .ok_or_else(|| "缺少驱动参数：action".to_string())?;
                peripheral_desk_protocol_razer_keyboard_light::apply_setting(KEY, PID, TID, action)
            }
            Some(operation) => Err(format!("Huntsman V2 TKL 不支持操作：{operation}")),
            None => Err("缺少驱动操作类型".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("razer-huntsman-v2-tkl", handler);
