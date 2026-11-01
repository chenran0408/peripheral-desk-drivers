//! Manual K99 V2 claim: the receiver ID also identifies other keyboards.
use peripheral_desk::{hid, keyboard::Keyboard, model::State};
use std::time::{SystemTime, UNIX_EPOCH};

const KEY: &str = "mchose-k99-v2";

fn read_state() -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x3554
                && i.pid == 0xfa09
                && i.usage_page == 0xff02
                && i.input_len == 20
                && i.output_len == 20
        })
        .collect();
    let [interface] = interfaces.as_slice() else {
        return Err("未找到唯一的 K99 V2 共用接收器键盘通道".into());
    };
    let block = Keyboard::open(&interface.path)?.read_block(0x4a)?;
    if block.len() != 2 || !(2..=100).contains(&block[0]) {
        return Err("键盘电量回复无效（0–1 为协议占位值）".into());
    }
    let charging = match block[1] >> 4 {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(block[0]),
        battery_raw: Some(block[0]),
        connection_path: Some("2.4 GHz 共用接收器".into()),
        charging,
        identity: "MCHOSE K99 V2（用户手动确认型号）".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("K99 V2 插件仅提供电量与已知充电状态读取".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("mchose-k99-v2", handler);
