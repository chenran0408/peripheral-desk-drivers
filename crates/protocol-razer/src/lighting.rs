//! Shared write-only Razer lighting editor contract.
//!
//! Mode and RGB state have no upstream read command for these devices. Only
//! brightness is read back; `saved` is the last configuration sent by this app.
use peripheral_desk::storage;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    BasiliskV3Pro,
    LogoMatrix,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectPacket {
    pub class: u8,
    pub command: u8,
    pub size: u8,
    pub args: Vec<u8>,
}

fn cache_path(config_dir: &str) -> Result<PathBuf, String> {
    let dir = Path::new(config_dir);
    if !dir.is_absolute() {
        return Err("灯光配置目录必须是绝对路径".into());
    }
    Ok(dir.join("lighting-sent.json"))
}

fn load_cache(config_dir: &str) -> Result<Value, String> {
    let path = cache_path(config_dir)?;
    match std::fs::read(&path) {
        Ok(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("已保存的灯光配置无效，未执行设置：{e}"))?;
            if !value.is_object() {
                return Err("已保存的灯光配置格式未知，未执行设置".into());
            }
            Ok(value)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(error) => Err(format!("读取已保存灯光配置失败：{error}")),
    }
}

fn save_cache(config_dir: &str, patch: Value) -> Result<(), String> {
    storage::write_json_preserving(&cache_path(config_dir)?, patch)
}

fn cached_config(cache: &Value) -> Option<Value> {
    let value = cache.get("saved")?;
    validate_config(value, None).ok()?;
    Some(value.clone())
}

fn validate_config(config: &Value, family: Option<Family>) -> Result<(), String> {
    let Some(enabled) = config.get("enabled").and_then(Value::as_bool) else {
        return Err("灯光配置缺少 enabled 布尔值".into());
    };
    let Some(brightness) = config.get("brightness").and_then(Value::as_u64) else {
        return Err("灯光配置缺少 brightness 值".into());
    };
    if brightness > 255 || (enabled && brightness == 0) {
        return Err("灯光亮度必须为 1–255；关闭灯光请使用 enabled=false".into());
    }
    let Some(effect) = config.get("effect").and_then(Value::as_u64) else {
        return Err("灯光配置缺少 effect 值".into());
    };
    if let Some(family) = family {
        let valid = match family {
            Family::BasiliskV3Pro => effect <= 2,
            Family::LogoMatrix => effect <= 4,
        };
        if !valid {
            return Err("此型号不支持所选灯效".into());
        }
    }
    let Some(rgb) = config.get("rgb").and_then(Value::as_array) else {
        return Err("灯光配置缺少 rgb 三通道数组".into());
    };
    if rgb.len() != 3
        || rgb
            .iter()
            .any(|value| value.as_u64().is_none_or(|v| v > 255))
    {
        return Err("rgb 必须包含 3 个 0–255 通道值".into());
    }
    let Some(speed) = config.get("speed").and_then(Value::as_u64) else {
        return Err("灯光配置缺少 speed 值".into());
    };
    if speed > 4 {
        return Err("灯效速度必须为 0–4".into());
    }
    if config.get("music_sync").and_then(Value::as_bool) != Some(false) {
        return Err("此型号不支持音乐同步".into());
    }
    Ok(())
}

fn suggested_config(family: Family, brightness: u8, enabled: bool) -> Value {
    json!({
        "enabled": enabled,
        "effect": 0,
        "rgb": [255, 255, 255],
        "speed": 1,
        "brightness": brightness.max(1),
        "music_sync": false,
        "suggested": true,
        "suggested_effects": match family {
            Family::BasiliskV3Pro => [0, 1, 2],
            Family::LogoMatrix => [0, 1, 2],
        },
    })
}

/// Builds honest state: only the brightness and enabled status are device readbacks.
pub fn read(config_dir: &str, brightness: u8, family: Family) -> Result<Value, String> {
    let cache = load_cache(config_dir)?;
    let saved = cached_config(&cache);
    let draft = saved
        .clone()
        .unwrap_or_else(|| suggested_config(family, brightness.max(128), brightness > 0));
    Ok(json!({
        "enabled": brightness > 0,
        "brightness": brightness,
        "brightness_readback": true,
        "readback": false,
        "saved": saved,
        "draft": draft,
        "notice": if saved.is_some() {
            "亮度来自设备读回；saved 是此应用上次成功发送的设置，设备当前灯效和颜色无法读取。"
        } else {
            "亮度来自设备读回；没有应用保存的灯效。draft 只是建议值，设备当前灯效和颜色无法读取。"
        }
    }))
}

/// Encodes a mode write using the pinned OpenRazer extended-matrix commands.
pub fn effect_packet(
    config: &Value,
    family: Family,
    led_id: u8,
) -> Result<Option<EffectPacket>, String> {
    validate_config(config, Some(family))?;
    if !config["enabled"].as_bool().unwrap_or(false) {
        return Ok(None);
    }
    let effect = config["effect"].as_u64().unwrap_or_default();
    let rgb: Vec<u8> = config["rgb"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    let speed = config["speed"].as_u64().unwrap_or(1).clamp(1, 4) as u8;
    let base = |size, effect_id| {
        let mut args = vec![1, led_id, effect_id, 0, 0, 0];
        args.resize(size as usize, 0);
        args
    };
    let (size, args) = match (family, effect) {
        (_, 0) => {
            let mut args = base(9, 1);
            args[5..9].copy_from_slice(&[1, rgb[0], rgb[1], rgb[2]]);
            (9, args)
        }
        (_, 1) => (6, base(6, 3)),
        (Family::BasiliskV3Pro, 2) => {
            let mut args = base(6, 4);
            args[3] = 0;
            args[4] = 0x28;
            (6, args)
        }
        (Family::LogoMatrix, 2) => {
            let mut args = base(9, 5);
            args[4..9].copy_from_slice(&[speed, 1, rgb[0], rgb[1], rgb[2]]);
            (9, args)
        }
        (Family::LogoMatrix, 3) => {
            let mut args = base(9, 2);
            args[3..9].copy_from_slice(&[1, 0, 1, rgb[0], rgb[1], rgb[2]]);
            (9, args)
        }
        (Family::LogoMatrix, 4) => (6, base(6, 2)),
        _ => return Err("此型号不支持所选灯效".into()),
    };
    Ok(Some(EffectPacket {
        class: 0x0f,
        command: 0x02,
        size,
        args,
    }))
}

fn send_cached<FEffect>(
    cache: &Value,
    family: Family,
    led_id: u8,
    send_effect: &mut FEffect,
) -> Option<String>
where
    FEffect: FnMut(&EffectPacket) -> Result<(), String>,
{
    let Some(config) = cached_config(cache) else {
        return Some("原设备灯效未知，且没有先前保存的设置可用于恢复".into());
    };
    if !config["enabled"].as_bool().unwrap_or(false) {
        return Some("之前保存的设置为关闭状态，未记录可恢复的灯效；原设备灯效未知".into());
    }
    match effect_packet(&config, family, led_id)
        .and_then(|packet| packet.map_or(Ok(()), |packet| send_effect(&packet)))
    {
        Ok(()) => None,
        Err(error) => Some(format!("恢复应用上次设置的灯效失败：{error}")),
    }
}

/// Applies a mode and brightness, rolling brightness back and restoring the last
/// known app-sent mode if a later step fails. Unknown device mode is never guessed.
pub fn apply<FRead, FWrite, FEffect>(
    config_dir: &str,
    config: &Value,
    family: Family,
    led_id: u8,
    mut read_brightness: FRead,
    mut write_brightness: FWrite,
    mut send_effect: FEffect,
) -> Result<Value, String>
where
    FRead: FnMut() -> Result<u8, String>,
    FWrite: FnMut(u8) -> Result<(), String>,
    FEffect: FnMut(&EffectPacket) -> Result<(), String>,
{
    validate_config(config, Some(family))?;
    let cache = load_cache(config_dir)?;
    let original = read_brightness().map_err(|e| format!("读取原灯光亮度失败，未发送设置：{e}"))?;
    let target = if config["enabled"].as_bool().unwrap_or(false) {
        config["brightness"].as_u64().unwrap() as u8
    } else {
        0
    };
    let mut mode_attempted = false;
    let mut failure = None;

    if target > 0 {
        if let Some(packet) = effect_packet(config, family, led_id)? {
            mode_attempted = true;
            if let Err(error) = send_effect(&packet) {
                failure = Some(format!("灯效写入未确认：{error}"));
            }
        }
    }
    if failure.is_none() && target != original {
        if let Err(error) = write_brightness(target) {
            failure = Some(format!("亮度写入未确认：{error}"));
        }
    }
    if failure.is_none() {
        std::thread::sleep(std::time::Duration::from_millis(180));
        match read_brightness() {
            Ok(actual) if actual == target => {}
            Ok(actual) => failure = Some(format!("亮度读回不匹配：期望 {target}，实际 {actual}")),
            Err(error) => failure = Some(format!("亮度读回失败：{error}")),
        }
    }
    if let Some(error) = failure {
        let mut rollback_errors = Vec::new();
        if target != original {
            if let Err(error) = write_brightness(original) {
                rollback_errors.push(format!("原亮度写入失败：{error}"));
            }
            std::thread::sleep(std::time::Duration::from_millis(180));
            match read_brightness() {
                Ok(actual) if actual == original => {}
                Ok(actual) => rollback_errors
                    .push(format!("原亮度读回不匹配：期望 {original}，实际 {actual}")),
                Err(error) => rollback_errors.push(format!("原亮度读回失败：{error}")),
            }
        }
        if mode_attempted {
            if let Some(error) = send_cached(&cache, family, led_id, &mut send_effect) {
                rollback_errors.push(error);
            }
        }
        return Err(if rollback_errors.is_empty() {
            format!("{error}；原亮度已恢复并读回确认。设备灯效没有读回接口，无法确认或恢复未知的原灯效。")
        } else {
            format!(
                "{error}；恢复结果未完全确认：{}",
                rollback_errors.join("；")
            )
        });
    }

    let mut patch = json!({"saved": config});
    if target > 0 {
        patch["last_nonzero_brightness"] = json!(target);
    }
    if let Err(error) = save_cache(config_dir, patch) {
        let mut rollback_errors = Vec::new();
        if target != original {
            if let Err(rollback_error) = write_brightness(original) {
                rollback_errors.push(format!("原亮度写入失败：{rollback_error}"));
            }
            std::thread::sleep(std::time::Duration::from_millis(180));
            match read_brightness() {
                Ok(actual) if actual == original => {}
                Ok(actual) => rollback_errors
                    .push(format!("原亮度读回不匹配：期望 {original}，实际 {actual}")),
                Err(rollback_error) => {
                    rollback_errors.push(format!("原亮度读回失败：{rollback_error}"))
                }
            }
        }
        if mode_attempted {
            if let Some(rollback_error) = send_cached(&cache, family, led_id, &mut send_effect) {
                rollback_errors.push(rollback_error);
            }
        }
        return Err(if rollback_errors.is_empty() {
            format!(
                "本地记录保存失败：{error}；原亮度已恢复并读回确认。设备原灯效未知，无法完整恢复。"
            )
        } else {
            format!(
                "本地记录保存失败：{error}；恢复未完全确认：{}",
                rollback_errors.join("；")
            )
        });
    }
    let saved = config.clone();
    Ok(json!({
        "enabled": target > 0,
        "brightness": target,
        "brightness_readback": true,
        "readback": false,
        "saved": saved,
        "draft": saved,
        "notice": "设备已接受灯效写入且亮度读回匹配；当前灯效和颜色无法从设备读取。saved 仅表示此应用上次成功发送的设置。"
    }))
}

/// Quick toggle changes brightness only, preserving the current unknown effect.
pub fn switch<FRead, FWrite>(
    config_dir: &str,
    enabled: bool,
    family: Family,
    mut read_brightness: FRead,
    mut write_brightness: FWrite,
) -> Result<Value, String>
where
    FRead: FnMut() -> Result<u8, String>,
    FWrite: FnMut(u8) -> Result<(), String>,
{
    let mut cache = load_cache(config_dir)?;
    let original = read_brightness().map_err(|e| format!("读取原灯光亮度失败，未发送设置：{e}"))?;
    let remembered = cache
        .get("last_nonzero_brightness")
        .and_then(Value::as_u64)
        .filter(|value| (1..=255).contains(value))
        .or_else(|| {
            cache
                .get("saved")
                .and_then(|value| value.get("brightness"))
                .and_then(Value::as_u64)
                .filter(|value| (1..=255).contains(value))
        })
        .unwrap_or(128) as u8;
    let target = if enabled { remembered } else { 0 };
    if target != original {
        if let Err(error) = write_brightness(target) {
            let _ = write_brightness(original);
            return Err(format!(
                "亮度切换未确认：{error}；已尝试恢复原亮度，原灯效保持不变。"
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(180));
    }
    let actual = match read_brightness() {
        Ok(actual) if actual == target => actual,
        Ok(actual) => {
            let _ = write_brightness(original);
            std::thread::sleep(std::time::Duration::from_millis(180));
            return Err(format!("亮度切换读回不匹配：期望 {target}，实际 {actual}；已尝试恢复原亮度，原灯效保持不变。"));
        }
        Err(error) => {
            let _ = write_brightness(original);
            std::thread::sleep(std::time::Duration::from_millis(180));
            return Err(format!(
                "亮度切换读回失败：{error}；已尝试恢复原亮度，原灯效保持不变。"
            ));
        }
    };
    if actual > 0 {
        cache["last_nonzero_brightness"] = json!(actual);
        save_cache(config_dir, json!({"last_nonzero_brightness": actual}))?;
    }
    let saved = cached_config(&cache);
    Ok(json!({
        "enabled": actual > 0,
        "brightness": actual,
        "brightness_readback": true,
        "readback": false,
        "saved": saved,
        "draft": saved.unwrap_or_else(|| suggested_config(family, remembered, actual > 0)),
        "notice": "仅切换亮度，设备原灯效和颜色未改动且无法读取；没有已保存灯效时 draft 仅为建议值。"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(_family: Family, effect: u8) -> Value {
        json!({"enabled":true,"effect":effect,"rgb":[10,20,30],"speed":2,"brightness":128,"music_sync":false})
    }

    #[test]
    fn effects_use_exact_upstream_matrix_payloads() {
        let static_config = config(Family::BasiliskV3Pro, 0);
        let packet = effect_packet(&static_config, Family::BasiliskV3Pro, 0)
            .unwrap()
            .unwrap();
        assert_eq!((packet.class, packet.command, packet.size), (0x0f, 0x02, 9));
        assert_eq!(packet.args, [1, 0, 1, 0, 0, 1, 10, 20, 30]);

        let wave = effect_packet(&config(Family::BasiliskV3Pro, 2), Family::BasiliskV3Pro, 0)
            .unwrap()
            .unwrap();
        assert_eq!(wave.args, [1, 0, 4, 0, 0x28, 0]);

        let logo = effect_packet(&config(Family::LogoMatrix, 2), Family::LogoMatrix, 4)
            .unwrap()
            .unwrap();
        assert_eq!(logo.args, [1, 4, 5, 0, 2, 1, 10, 20, 30]);
        let breath = effect_packet(&config(Family::LogoMatrix, 3), Family::LogoMatrix, 4)
            .unwrap()
            .unwrap();
        assert_eq!(breath.args, [1, 4, 2, 1, 0, 1, 10, 20, 30]);
        let spectrum = effect_packet(&config(Family::LogoMatrix, 1), Family::LogoMatrix, 4)
            .unwrap()
            .unwrap();
        assert_eq!(spectrum.args, [1, 4, 3, 0, 0, 0]);
        let random_breath = effect_packet(&config(Family::LogoMatrix, 4), Family::LogoMatrix, 4)
            .unwrap()
            .unwrap();
        assert_eq!(random_breath.args, [1, 4, 2, 0, 0, 0]);
    }
}
