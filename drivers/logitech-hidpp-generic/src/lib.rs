use peripheral_desk::hid::{self, Interface};
use peripheral_desk_protocol_hidpp::{read_shared_devices, transport::WinTransport};
use serde_json::Value;

const KEY: &str = "logitech-hidpp-generic";
const RECEIVERS: &[u16] = &[
    0xc52b, 0xc532, // Unifying
    0xc539, 0xc53a, 0xc53d, 0xc53f, 0xc541, 0xc545, 0xc547, 0xc54d, // Lightspeed
    0xc548, // Bolt
];
const WIRED: &[u16] = &[0xc09b];

fn long_interface(i: &Interface) -> bool {
    i.vid == 0x046d
        && (RECEIVERS.contains(&i.pid) || WIRED.contains(&i.pid))
        && i.usage_page == 0xff00
        && i.usage == 2
        && i.input_len == 20
        && i.output_len == 20
}

fn short_for<'a>(interfaces: &'a [Interface], long: &Interface) -> Option<&'a Interface> {
    let path = long.path.to_ascii_lowercase();
    let (prefix, collection) = path.split_once("&col")?;
    let (_, instance) = collection.split_once('#')?;
    let mut matches = interfaces.iter().filter(|i| {
        let path = i.path.to_ascii_lowercase();
        let same_instance = path
            .split_once("&col")
            .and_then(|(candidate_prefix, collection)| {
                collection.split_once('#').map(|(_, candidate_instance)| {
                    candidate_prefix == prefix && candidate_instance == instance
                })
            })
            .unwrap_or(false);
        i.vid == long.vid
            && i.pid == long.pid
            && i.usage_page == 0xff00
            && i.usage == 1
            && i.input_len == 7
            && i.output_len == 7
            && same_instance
    });
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

fn read_devices() -> Result<Vec<peripheral_desk::model::State>, String> {
    let interfaces = hid::enumerate()?;
    let mut found = Vec::new();
    for interface in interfaces.iter().filter(|i| long_interface(i)) {
        let Ok(io) = WinTransport::open(&interface.path) else {
            continue;
        };
        let short =
            short_for(&interfaces, interface).and_then(|i| WinTransport::open(&i.path).ok());
        found.extend(read_shared_devices(
            io,
            short,
            interface.pid,
            &interface.product,
            RECEIVERS.contains(&interface.pid),
        ));
    }
    Ok(found)
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if key != KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let req: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        let mut devices = match req.get("op").and_then(Value::as_str) {
            Some("read-state" | "read-devices") => read_devices()?,
            _ => return Err("此插件仅支持读取电量".into()),
        };
        if req.get("op").and_then(Value::as_str) == Some("read-devices") {
            return serde_json::to_value(devices).map_err(|e| e.to_string());
        }
        let mut first = devices
            .drain(..)
            .next()
            .ok_or("没有已连接且提供有效电量的罗技设备")?;
        first.key = KEY.into();
        serde_json::to_value(first).map_err(|e| e.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("logitech-hidpp-generic", handler);
