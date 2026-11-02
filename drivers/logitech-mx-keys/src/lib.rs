use peripheral_desk_protocol_hidpp::transport::WinTransport;
use serde_json::Value;

const KEY: &str = "logitech-mx-keys";
const PID: u16 = 0xB35B;
const WPID: u16 = 0x408A;
const MODEL_NAME: &str = "MX Keys Wireless Keyboard";

fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if key != KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let req: Value = serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        if req.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("该插件仅提供蓝牙电量读取".into());
        }
        let interfaces = peripheral_desk::hid::enumerate()?;
        let mut last =
            format!("未找到 Logitech MX Keys Keyboard 蓝牙 046D:{PID:04X} HID++ 长报文接口");
        for i in interfaces.iter().filter(|i| {
            i.vid == 0x046d
                && i.pid == PID
                && i.usage_page == 0xff00
                && i.usage == 2
                && i.input_len == 20
                && i.output_len == 20
        }) {
            match WinTransport::open(&i.path).and_then(|t| {
                peripheral_desk_protocol_hidpp::read_logitech_battery(t, WPID, MODEL_NAME, KEY)
            }) {
                Ok(state) => return serde_json::to_value(state).map_err(|e| e.to_string()),
                Err(e) => last = e,
            }
        }
        Err(last)
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("logitech-mx-keys", handler);
