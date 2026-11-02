use peripheral_desk_protocol_scyrox::{read_state, Model};

const MODEL: Model = Model {
    key: "scyrox-v8s",
    name: "Scyrox V8S",
    wired_pid: 0xf515,
    receiver_pid: 0xf517,
};

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != MODEL.key {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("Scyrox V8S 插件仅支持只读电量查询".into());
        }
        serde_json::to_value(read_state(MODEL)?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("scyrox-v8s", handler);
