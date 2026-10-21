use peripheral_desk_protocol_mchose_magdock::{self as magdock, Lighting};
use serde_json::{json, Value};

fn config(value: &Value) -> Result<Lighting, String> {
    let field = |name: &str| {
        value
            .get(name)
            .ok_or_else(|| format!("缺少灯效参数：{name}"))
    };
    let boolean = |name: &str| -> Result<bool, String> {
        field(name)?
            .as_bool()
            .ok_or_else(|| format!("灯效参数 {name} 必须是布尔值"))
    };
    let byte = |name: &str, maximum: u64| -> Result<u8, String> {
        let number = field(name)?
            .as_u64()
            .filter(|number| *number <= maximum)
            .ok_or_else(|| format!("灯效参数 {name} 超出允许范围 0–{maximum}"))?;
        Ok(number as u8)
    };
    let rgb_value = field("rgb")?;
    let rgb_array = rgb_value
        .as_array()
        .filter(|rgb| rgb.len() == 3)
        .ok_or("灯效参数 rgb 必须包含三个 0–255 数值")?;
    let mut rgb = [0; 3];
    for (index, channel) in rgb_array.iter().enumerate() {
        rgb[index] = channel
            .as_u64()
            .filter(|number| *number <= 255)
            .ok_or("灯效参数 rgb 必须包含三个 0–255 数值")? as u8;
    }
    let effect = byte("effect", 5)?;
    let music_sync = boolean("music_sync")?;
    if music_sync != (effect == 5) {
        return Err("MagDock 的 Music 效果与音乐同步开关必须保持一致".into());
    }
    Ok(Lighting {
        enabled: boolean("enabled")?,
        effect,
        effect_count: 0,
        speed: byte("speed", 4)?,
        brightness: byte("brightness", 4)?,
        music_sync,
        rgb,
        direction: 0,
    })
}

fn state(lighting: &Lighting, notice: String) -> Value {
    json!({
        "enabled": lighting.enabled,
        "saved": {
            "enabled": lighting.enabled,
            "effect": lighting.effect,
            "rgb": lighting.rgb,
            "speed": lighting.speed,
            "brightness": lighting.brightness,
            "music_sync": lighting.music_sync
        },
        "notice": notice,
        "readback": true
    })
}

fn handler(device_key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if device_key != magdock::DEVICE_KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let request: Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        match request.get("op").and_then(Value::as_str) {
            Some("read-state") => {
                serde_json::to_value(magdock::read_state()?).map_err(|e| e.to_string())
            }
            Some("lighting-read") => {
                let block = magdock::read_lighting()?;
                Ok(state(&block.lighting, "灯效状态来自设备实际读回；上游 OpenMouse 已在 MagDock 上完成写入、核对和恢复记录，本项目尚未实机验证。".into()))
            }
            Some("lighting-apply") => {
                let wanted = config(request.get("config").ok_or("缺少驱动参数：config")?)?;
                let readback = magdock::apply_lighting(&wanted)?;
                Ok(state(&readback.lighting, "灯效已写入并从设备读回核对；未匹配时驱动会尝试恢复原始灯效。本项目尚未在本机实机验证。".into()))
            }
            Some("lighting-switch") => {
                let enabled = request
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .ok_or("灯效参数 enabled 必须是布尔值")?;
                let readback = magdock::switch_lighting(enabled)?;
                Ok(state(
                    &readback.lighting,
                    "开关状态来自设备实际读回；底座其他灯效值保留。本项目尚未实机验证。".into(),
                ))
            }
            _ => {
                Err("MCHOSE MagDock 仅提供 lighting-read、lighting-apply 和 lighting-switch".into())
            }
        }
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}).to_string(),
        Err(error) => json!({"ok":false,"value":null,"error":error}).to_string(),
    }
}

peripheral_desk_driver_ffi::export_driver!("mchose-magdock", handler);
