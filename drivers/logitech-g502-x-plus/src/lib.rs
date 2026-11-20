//! Exact-model LIGHTSPEED battery and onboard selection.
#[path = "../../logitech-g502-lightspeed/src/battery.rs"]
mod battery;
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "logitech-g502-x-plus" {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request["op"]
            .as_str()
            .is_some_and(peripheral_desk_protocol_hidpp::onboard::handles)
        {
            return peripheral_desk_protocol_hidpp::onboard::handle(key, &request);
        }
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("此插件支持电量读取和板载配置选择".into());
        }
        serde_json::to_value(battery::read_state(key, "G502 X PLUS")?)
            .map_err(|err| err.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("logitech-g502-x-plus", handler);
