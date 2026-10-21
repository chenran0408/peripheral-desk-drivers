fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_mchose_v3::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => {
                serde_json::to_value(peripheral_desk_protocol_mchose_v3::read_state()?)
                    .map_err(|e| e.to_string())
            }
            Some("apply-setting") => {
                let action: peripheral_desk::model::Action =
                    serde_json::from_value(request.get("action").cloned().ok_or("缺少设置操作")?)
                        .map_err(|e| format!("设置操作无效：{e}"))?;
                peripheral_desk_protocol_mchose_v3::apply_action(action)?;
                Ok(serde_json::Value::Null)
            }
            Some("controls-read") => {
                serde_json::to_value(peripheral_desk_protocol_mchose_v3::read_controls_state()?)
                    .map_err(|e| e.to_string())
            }
            Some("controls-apply") => {
                let action: peripheral_desk::controls::ControlAction =
                    serde_json::from_value(request.get("action").cloned().ok_or("缺少键位操作")?)
                        .map_err(|e| format!("键位操作无效：{e}"))?;
                serde_json::to_value(peripheral_desk_protocol_mchose_v3::apply_control_action(
                    &action,
                )?)
                .map_err(|e| e.to_string())
            }
            _ => Err("MCHOSE A7 V3 Ultra+ 预览支持状态读取、DPI/回报率设置和按键映射".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("mchose-a7-v3-ultra-plus", handler);
