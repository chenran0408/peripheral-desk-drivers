use peripheral_desk::{hid::Interface, model::State};
use peripheral_desk_protocol_atk_f1_battery::{is_atk_page, read_state_prefer};

const KEY: &str = "atk-f1-v3-ultimate-plus";

fn supported(i: &Interface) -> bool {
    if !is_atk_page(i) {
        return false;
    }
    match i.pid {
        // Vigil Watt v1.3.0 documents this exact 8K-M receiver interface.
        0x1303 => {
            i.input_len == 17
                && i.output_len == 17
                && i.path.to_ascii_lowercase().contains("&mi_01&col05#")
        }
        // The wired body is documented with this exact product identity, but
        // its report descriptor lengths were not measured upstream.
        0x12a9 => {
            i.input_len >= 17
                && i.output_len >= 17
                && i.product.eq_ignore_ascii_case("ATK F1 V3 ULTIMATE+")
        }
        _ => false,
    }
}

fn prefer_receiver(i: &Interface) -> bool {
    i.pid == 0x1303
}

fn read() -> Result<State, String> {
    read_state_prefer(KEY, "ATK F1 V3 ULTIMATE+", supported, prefer_receiver)
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
        read()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("atk-f1-v3-ultimate-plus", handler);
