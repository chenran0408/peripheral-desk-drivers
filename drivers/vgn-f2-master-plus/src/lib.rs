use peripheral_desk_protocol_vgn::DEVICE_KEY;

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err(
                "VGN Dragonfly F2 Master+ 预览仅提供只读电量、充电状态和固件版本；没有设置写入操作"
                    .into(),
            );
        }
        serde_json::to_value(peripheral_desk_protocol_vgn::read_vgn()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("vgn-f2-master-plus", handler);
