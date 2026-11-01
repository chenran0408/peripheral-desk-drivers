const SPEC: peripheral_desk_protocol_wlmouse::Spec = peripheral_desk_protocol_wlmouse::Spec {
    key: "wlmouse-beast-x-mini-pro",
    name: "WLmouse Beast X Mini Pro",
    pid: 0xa868,
};
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != SPEC.key {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("只支持读取电量".into());
        }
        serde_json::to_value(peripheral_desk_protocol_wlmouse::read_state(&SPEC)?)
            .map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("wlmouse-beast-x-mini-pro", handler);
