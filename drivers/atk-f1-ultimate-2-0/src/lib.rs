use peripheral_desk::hid::Interface;
use peripheral_desk_protocol_atk_f1_battery::{is_atk_page, read_state};

const KEY: &str = "atk-f1-ultimate-2-0";

fn supported(i: &Interface) -> bool {
    is_atk_page(i) && matches!(i.pid, 0x11d9 | 0x11e4) && i.input_len >= 17 && i.output_len >= 17
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("此插件只读取电量与充电状态，不执行设置写入".to_string());
        }
        read_state(KEY, "ATK F1 Ultimate 2.0", supported)
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("atk-f1-ultimate-2-0", handler);
