//! Exact-model G305 battery and guarded DPI preview; no registry-wide matching.
use peripheral_desk_protocol_hidpp::{apply_g305_setting, read_g305, G305_KEY};
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != G305_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_g305()?).map_err(|e| e.to_string()),
            Some("apply-setting") => {
                let action: peripheral_desk_protocol_hidpp::Action = serde_json::from_value(
                    request.get("action").cloned().ok_or("缺少 G305 设置动作")?,
                )
                .map_err(|e| format!("G305 设置动作无效：{e}"))?;
                apply_g305_setting(action)?;
                Ok(serde_json::Value::Null)
            }
            _ => Err("G305 支持只读状态和精确型号 DPI 设置；改键、宏、回报率和灯光未开放".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("logitech-g305", handler);
