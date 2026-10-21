use peripheral_desk_protocol_vgn::DEVICE_KEY;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(peripheral_desk_protocol_vgn::read_vgn()?)
                .map_err(|e| e.to_string()),
            Some("apply-setting") => {
                let action: peripheral_desk_protocol_vgn::Action = serde_json::from_value(
                    request.get("action").cloned().ok_or("缺少 VGN 设置动作")?,
                )
                .map_err(|e| format!("VGN 设置动作无效：{e}"))?;
                peripheral_desk_protocol_vgn::apply_vgn_setting(action)?;
                Ok(serde_json::Value::Null)
            }
            Some("controls-read") => {
                serde_json::to_value(peripheral_desk_protocol_vgn::read_vgn_controls()?)
                    .map_err(|e| e.to_string())
            }
            Some("controls-apply") => {
                let action: peripheral_desk_protocol_vgn::ControlAction = serde_json::from_value(
                    request.get("action").cloned().ok_or("缺少 VGN 按键动作")?,
                )
                .map_err(|e| format!("VGN 按键动作无效：{e}"))?;
                peripheral_desk_protocol_vgn::apply_vgn_controls(&action)?;
                serde_json::to_value(peripheral_desk_protocol_vgn::read_vgn_controls()?)
                    .map_err(|e| e.to_string())
            }
            _ => Err("VGN Dragonfly F2 Master+ preview supports read-state, DPI/rate settings, and six-button remapping only; no lighting, macros, or firmware commands are available".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("vgn-f2-master-plus", handler);
