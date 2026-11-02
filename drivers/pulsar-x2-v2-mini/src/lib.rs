const SPEC: peripheral_desk_protocol_pulsar_17::Spec = peripheral_desk_protocol_pulsar_17::Spec {
    key: "pulsar-x2-v2-mini",
    name: "Pulsar X2 V2 Mini",
    vid: 0x3554,
    receiver: 0xf508,
    wired: Some((0x3554, 0xf507)),
};

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != SPEC.key {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("Pulsar X2 V2 Mini 插件仅支持只读电量查询".into());
        }
        serde_json::to_value(peripheral_desk_protocol_pulsar_17::read_state(&SPEC)?)
            .map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("pulsar-x2-v2-mini", handler);
