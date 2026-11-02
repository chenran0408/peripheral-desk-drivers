//! Razer keyboard backlight switching, based on OpenRazer's matrix brightness commands.
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::Action,
    storage,
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

const VID: u16 = 0x1532;
const REPORT_LEN: usize = 91;
const LED: u8 = 0x05;
const LIGHT_READ: (u8, u8, u8) = (0x0f, 0x84, 3);
const LIGHT_WRITE: (u8, u8, u8) = (0x0f, 0x04, 3);
const WAIT: Duration = Duration::from_millis(31);

fn matching_interface(interface: &Interface, pid: u16) -> bool {
    interface.vid == VID
        && interface.pid == pid
        && interface.usage_page == 0xff00
        && interface.feature_len == REPORT_LEN as u16
}

fn unique_device(pid: u16, key: &str) -> Result<Device, String> {
    let interfaces = hid::enumerate()?;
    let matches: Vec<_> = interfaces
        .iter()
        .filter(|i| matching_interface(i, pid))
        .collect();
    match matches.as_slice() {
        [interface] => Device::open(&interface.path),
        [] => Err(format!(
            "未找到 {key} 的唯一 Feature 接口（1532:{pid:04X} / FF00 / 91 字节）"
        )),
        _ => Err(format!(
            "发现多个 {key} Feature 接口；为避免操作错误设备，已停止"
        )),
    }
}

fn checksum(wire: &[u8; REPORT_LEN]) -> u8 {
    wire[3..89].iter().fold(0, |sum, byte| sum ^ byte)
}

fn request(tid: u8, command: (u8, u8, u8), args: &[u8]) -> Result<[u8; REPORT_LEN], String> {
    if args.len() > command.2 as usize || 9 + command.2 as usize > 89 {
        return Err("Razer 灯光命令参数长度无效".into());
    }
    let mut wire = [0; REPORT_LEN];
    wire[2] = tid;
    wire[6] = command.2;
    wire[7] = command.0;
    wire[8] = command.1;
    wire[9..9 + args.len()].copy_from_slice(args);
    wire[89] = checksum(&wire);
    Ok(wire)
}

fn reply_body<'a>(wire: &'a [u8], req: &[u8; REPORT_LEN]) -> Result<&'a [u8], String> {
    let Ok(wire) = <&[u8; REPORT_LEN]>::try_from(wire) else {
        return Err("Razer Feature 回复长度不匹配".into());
    };
    if wire[0] != 0 || wire[89] != checksum(wire) {
        return Err("Razer Feature 回复 Report ID 或校验不匹配".into());
    }
    if wire[2] != req[2]
        || wire[3..5] != req[3..5]
        || wire[5] != 0
        || wire[6] != req[6]
        || wire[7] != req[7]
        || wire[8] != req[8]
    {
        return Err("Razer Feature 回复与请求不匹配".into());
    }
    match wire[1] {
        0x02 => Ok(&wire[9..9 + wire[6] as usize]),
        0x01 => Err("Razer 设备仍在处理灯光请求".into()),
        status => Err(format!("Razer 灯光请求失败，状态 0x{status:02X}")),
    }
}

fn send(device: &Device, tid: u8, command: (u8, u8, u8), args: &[u8]) -> Result<Vec<u8>, String> {
    let req = request(tid, command, args)?;
    device.send(&req)?;
    thread::sleep(WAIT);
    let response = device.receive(0, REPORT_LEN)?;
    reply_body(&response, &req).map(ToOwned::to_owned)
}

fn read_brightness(device: &Device, tid: u8) -> Result<u8, String> {
    send(device, tid, LIGHT_READ, &[1, LED])?
        .get(2)
        .copied()
        .ok_or_else(|| "Razer 背光亮度回复字段缺失".into())
}

fn write_brightness(device: &Device, tid: u8, brightness: u8) -> Result<(), String> {
    send(device, tid, LIGHT_WRITE, &[1, LED, brightness]).map(|_| ())
}

fn cache_path(config_dir: &str, key: &str) -> Result<PathBuf, String> {
    let dir = Path::new(config_dir);
    if !dir.is_absolute() {
        return Err("灯光配置目录必须是绝对路径".into());
    }
    Ok(dir.join(format!("{key}-lighting.json")))
}

fn remembered_brightness(path: &Path) -> Result<u8, String> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)
                .map_err(|error| format!("已保存的灯光配置无效：{error}"))?;
            match value.get("last_nonzero_brightness").and_then(Value::as_u64) {
                Some(value @ 1..=255) => Ok(value as u8),
                Some(_) => Err("已保存的背光亮度无效".into()),
                None => Ok(128),
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(128),
        Err(error) => Err(format!("读取已保存灯光配置失败：{error}")),
    }
}

fn save_brightness(path: &Path, brightness: u8) -> Result<(), String> {
    storage::write_json_preserving(path, json!({"last_nonzero_brightness": brightness}))
}

pub fn lighting_read(key: &str, pid: u16, tid: u8) -> Result<Value, String> {
    let device = unique_device(pid, key)?;
    let brightness = read_brightness(&device, tid)?;
    Ok(json!({
        "enabled": brightness > 0,
        "brightness": brightness,
        "brightness_readback": true,
        "readback": false,
        "saved": null,
        "draft": {"enabled": brightness > 0, "brightness": brightness.max(128)},
        "notice": "当前背光亮度来自设备读回；设备灯效和颜色无法读取。"
    }))
}

pub fn lighting_switch(
    key: &str,
    pid: u16,
    tid: u8,
    config_dir: &str,
    enabled: bool,
) -> Result<Value, String> {
    let path = cache_path(config_dir, key)?;
    let device = unique_device(pid, key)?;
    let original = read_brightness(&device, tid)
        .map_err(|error| format!("读取原背光亮度失败，未写入：{error}"))?;
    let remembered = if original > 0 {
        original
    } else {
        remembered_brightness(&path)?
    };
    let target = if enabled { remembered } else { 0 };
    if original != target {
        write_brightness(&device, tid, target)
            .map_err(|error| format!("背光写入未确认：{error}"))?;
        thread::sleep(Duration::from_millis(180));
    }
    let actual = read_brightness(&device, tid)?;
    if actual != target {
        return Err(format!("背光读回不匹配：期望 {target}，实际 {actual}"));
    }
    save_brightness(&path, remembered)?;
    Ok(json!({
        "enabled": actual > 0,
        "brightness": actual,
        "brightness_readback": true,
        "readback": false,
        "saved": null,
        "draft": {"enabled": actual > 0, "brightness": actual.max(128)},
        "notice": "仅切换背光亮度；设备灯效和颜色未修改且无法读取。"
    }))
}

pub fn apply_setting(key: &str, pid: u16, tid: u8, action: &Value) -> Result<Value, String> {
    let action: Action = serde_json::from_value(action.clone())
        .map_err(|error| format!("驱动参数 action 无效：{error}"))?;
    let Action::Light(brightness) = action else {
        return Err("此型号只接受 Action::Light(u8) 背光设置".into());
    };
    let device = unique_device(pid, key)?;
    write_brightness(&device, tid, brightness)?;
    thread::sleep(Duration::from_millis(180));
    match read_brightness(&device, tid)? {
        actual if actual == brightness => Ok(Value::Null),
        actual => Err(format!(
            "背光亮度读回不匹配：期望 {brightness}，实际 {actual}"
        )),
    }
}
