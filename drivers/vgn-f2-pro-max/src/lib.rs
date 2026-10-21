fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != peripheral_desk_protocol_vgn::F2_PRO_MAX.key {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err(
                "VGN Dragonfly F2 Pro Max 预览仅提供只读电量和充电状态；没有设置写入操作".into(),
            );
        }
        serde_json::to_value(peripheral_desk_protocol_vgn::read_vgn_for(
            peripheral_desk_protocol_vgn::F2_PRO_MAX,
        )?)
        .map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("vgn-f2-pro-max", handler);
