//! Razer Viper V4 Pro Windows Feature-report driver.
//!
//! Battery framing follows Vigil Watt v1.3.0. DPI, polling and sleep
//! operations are adapted from OpenMouse mouse-protocol commit
//! 23f5bed75d148cabeed19c74838deb31d3f60d02 (AGPL-3.0-or-later).
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::Action,
};
use serde_json::{json, Value};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const DEVICE_KEY: &str = "razer-viper-v4-pro";
const VID: u16 = 0x1532;
const WIRED_PID: u16 = 0x00e5;
const RECEIVER_PID: u16 = 0x00e6;
const FEATURE_LEN: u16 = 91;
const REPORT_ID: u8 = 0;
const TID: u8 = 0x1f;
const WAIT: Duration = Duration::from_millis(35);
const POLLING: [(u16, u8); 6] = [
    (125, 0x40),
    (500, 0x10),
    (1000, 0x08),
    (2000, 0x04),
    (4000, 0x02),
    (8000, 0x01),
];
const SLEEP: [u16; 4] = [60, 300, 600, 900];

fn matches(i: &Interface) -> bool {
    i.vid == VID
        && matches!(i.pid, WIRED_PID | RECEIVER_PID)
        && i.feature_len == FEATURE_LEN
        && matches!(i.usage_page, 0x0001 | 0x000c)
}

fn open_unique() -> Result<(Device, u16), String> {
    let interfaces: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    match interfaces.as_slice() {
        [i] => Ok((Device::open(&i.path)?, i.pid)),
        [] => Err("未找到 Viper V4 Pro 的经典 91 字节 Feature 接口（1532:00E5/00E6）".into()),
        _ => Err("检测到多个 Viper V4 Pro Feature 接口；为避免写错设备，已停止".into()),
    }
}

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |sum, byte| sum ^ byte)
}

fn request(
    class: u8,
    command: u8,
    size: u8,
    args: &[u8],
) -> Result<[u8; FEATURE_LEN as usize], String> {
    if usize::from(size) > 80 || args.len() > usize::from(size) {
        return Err("Viper V4 Pro 请求长度超出协议范围".into());
    }
    let mut wire = [0; FEATURE_LEN as usize];
    wire[0] = REPORT_ID;
    wire[2] = TID;
    wire[6] = size;
    wire[7] = class;
    wire[8] = command;
    wire[9..9 + args.len()].copy_from_slice(args);
    wire[89] = checksum(&wire[3..89]);
    Ok(wire)
}

fn exchange(
    device: &Device,
    class: u8,
    command: u8,
    size: u8,
    args: &[u8],
) -> Result<Vec<u8>, String> {
    let req = request(class, command, size, args)?;
    device.send(&req)?;
    let mut last_error = String::from("Viper V4 Pro 持续返回忙状态");
    for _ in 0..16 {
        let response = device.receive(REPORT_ID, FEATURE_LEN as usize)?;
        if response.len() != FEATURE_LEN as usize {
            return Err("Viper V4 Pro Feature 回复长度不匹配".into());
        }
        if response[1] == 0x01 {
            last_error = "Viper V4 Pro 正忙，请稍后重试".into();
            thread::sleep(WAIT);
            continue;
        }
        if response[1] == 0x04 {
            return Err("Viper V4 Pro 未响应，可能处于休眠状态".into());
        }
        if response[1] != 0x02 {
            return Err(format!(
                "Viper V4 Pro 拒绝命令（状态 0x{:02X}）",
                response[1]
            ));
        }
        if response[0] != REPORT_ID
            || response[2] != TID
            || response[7] != class
            || response[8] != command
            || response[89] != checksum(&response[3..89])
        {
            return Err("Viper V4 Pro 回复校验或命令匹配失败".into());
        }
        let length = response[6] as usize;
        if length > 80 {
            return Err("Viper V4 Pro 回复长度字段无效".into());
        }
        return Ok(response[9..9 + length].to_vec());
    }
    Err(last_error)
}

#[derive(Clone, Debug)]
struct DpiState {
    active: usize,
    stages: Vec<(u16, u16)>,
}

fn read_dpi(device: &Device) -> Result<DpiState, String> {
    let raw = exchange(device, 0x04, 0x86, 80, &[])?;
    if raw.len() < 3 {
        return Err("Viper V4 Pro DPI 档位回复过短".into());
    }
    let count = raw[2].min(5) as usize;
    let active = raw[1].saturating_sub(1) as usize;
    if count == 0 || active >= count || raw.len() < 3 + count * 7 {
        return Err("Viper V4 Pro DPI 档位表结构不匹配".into());
    }
    let mut stages = Vec::with_capacity(count);
    for n in 0..count {
        let at = 3 + n * 7;
        let x = u16::from_be_bytes([raw[at + 1], raw[at + 2]]);
        let y = u16::from_be_bytes([raw[at + 3], raw[at + 4]]);
        if !(100..=50_000).contains(&x) || !(100..=50_000).contains(&y) {
            return Err("Viper V4 Pro DPI 值不在已知范围内".into());
        }
        stages.push((x, y));
    }
    Ok(DpiState { active, stages })
}

fn encode_dpi(active: usize, stages: &[(u16, u16)]) -> Vec<u8> {
    let mut args = vec![0; 3 + stages.len() * 7];
    args[0] = 1;
    args[1] = (active + 1) as u8;
    args[2] = stages.len() as u8;
    for (n, (x, y)) in stages.iter().enumerate() {
        let at = 3 + n * 7;
        args[at] = n as u8;
        args[at + 1..at + 3].copy_from_slice(&x.to_be_bytes());
        args[at + 3..at + 5].copy_from_slice(&y.to_be_bytes());
    }
    args
}

fn restore_dpi(device: &Device, old: &DpiState) -> Result<(), String> {
    let args = encode_dpi(old.active, &old.stages);
    exchange(device, 0x04, 0x06, args.len() as u8, &args)?;
    let restored = read_dpi(device)?;
    if restored.active != old.active || restored.stages != old.stages {
        return Err("原 DPI 档位表恢复后读回不匹配".into());
    }
    Ok(())
}

fn write_dpi(device: &Device, old: &DpiState, dpi: u16) -> Result<(), String> {
    if !(100..=50_000).contains(&dpi) || !dpi.is_multiple_of(50) {
        return Err("Viper V4 Pro DPI 必须为 100–50000，按 50 DPI 递增".into());
    }
    let mut stages = old.stages.clone();
    stages[old.active] = (dpi, dpi);
    let args = encode_dpi(old.active, &stages);
    if let Err(error) = exchange(device, 0x04, 0x06, args.len() as u8, &args) {
        return match restore_dpi(device, old) {
            Ok(()) => Err(format!("DPI 写入未确认：{error}；原档位表已恢复并读回")),
            Err(restore) => Err(format!(
                "DPI 写入未确认：{error}；原档位表恢复也未确认：{restore}"
            )),
        };
    }
    let confirmed = read_dpi(device);
    if !matches!(confirmed, Ok(ref value) if value.active == old.active && value.stages == stages) {
        let reason = match confirmed {
            Ok(_) => "设备读回的 DPI 档位与请求不一致".to_string(),
            Err(error) => format!("设备读回 DPI 失败：{error}"),
        };
        return match restore_dpi(device, old) {
            Ok(()) => Err(format!("{reason}；原档位表已恢复并读回")),
            Err(restore) => Err(format!("{reason}；原档位表恢复也未确认：{restore}")),
        };
    }
    Ok(())
}

fn percent(raw: u8) -> u8 {
    ((u16::from(raw) * 100) / 255) as u8
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn read_state(device: &Device, pid: u16) -> Result<Value, String> {
    let dpi = read_dpi(device)?;
    let rate = exchange(device, 0x00, 0xc0, 2, &[1])?;
    let battery = exchange(device, 0x07, 0x80, 2, &[0, 0])?;
    let charge = exchange(device, 0x07, 0x84, 2, &[0, 0]);
    let sleep = exchange(device, 0x07, 0x83, 2, &[])?;
    let low = exchange(device, 0x07, 0x81, 1, &[])?;
    if battery.len() < 2 || sleep.len() < 2 || low.is_empty() {
        return Err("Viper V4 Pro 状态回复缺少字段".into());
    }
    let raw = battery[1];
    let charging = charge.ok().and_then(|v| match v.get(1) {
        Some(0) => Some(false),
        Some(1) => Some(true),
        _ => None,
    });
    let current = dpi.stages[dpi.active];
    let hz = rate
        .get(1)
        .and_then(|code| POLLING.iter().find(|(_, c)| c == code).map(|(hz, _)| *hz));
    let seconds = u16::from_be_bytes([sleep[0], sleep[1]]);
    Ok(json!({
        "key": DEVICE_KEY,
        "present": true,
        "fresh": true,
        "percent": percent(raw),
        "battery_raw": raw,
        "connection_path": if pid == WIRED_PID { "USB 有线" } else { "HyperSpeed 接收器" },
        "charging": charging,
        "wired": pid == WIRED_PID,
        "dpi": current.0,
        "rate": hz,
        "sleep_seconds": seconds,
        "updated": now(),
        "identity": format!("Razer Viper V4 Pro · 1532:{pid:04X} · 91-byte Feature"),
        "error": "",
        "settings_error": "",
        "tuning": { "LowPower": percent(low[0]) as u16 },
        "dpi_stages": dpi.stages.iter().map(|(x, _)| *x).collect::<Vec<_>>(),
        "active_stage": dpi.active as u8
    }))
}

fn apply(device: &Device, action: Action) -> Result<(), String> {
    match action {
        Action::Dpi(dpi) => write_dpi(device, &read_dpi(device)?, dpi),
        Action::Rate(hz) => {
            let code = POLLING
                .iter()
                .find(|(rate, _)| *rate == hz)
                .map(|(_, code)| *code)
                .ok_or("Viper V4 Pro 不支持该回报率")?;
            let old_code = exchange(device, 0x00, 0xc0, 2, &[1])?
                .get(1)
                .copied()
                .ok_or("Viper V4 Pro 原回报率回复字段缺失")?;
            if let Err(error) = exchange(device, 0x00, 0x40, 2, &[1, code]) {
                return match restore_rate(device, old_code) {
                    Ok(()) => Err(format!("回报率写入未确认：{error}；原值已恢复并读回")),
                    Err(restore) => Err(format!(
                        "回报率写入未确认：{error}；原值恢复也未确认：{restore}"
                    )),
                };
            }
            thread::sleep(Duration::from_millis(150));
            let actual = match exchange(device, 0x00, 0xc0, 2, &[1]) {
                Ok(actual) => actual,
                Err(error) => {
                    let rollback = restore_rate(device, old_code);
                    return Err(match rollback {
                        Ok(()) => format!("回报率读回失败：{error}；原值已恢复并读回"),
                        Err(restore) => {
                            format!("回报率读回失败：{error}；原值恢复也未确认：{restore}")
                        }
                    });
                }
            };
            if actual.get(1) != Some(&code) {
                let rollback = restore_rate(device, old_code);
                return Err(match rollback {
                    Ok(()) => "Viper V4 Pro 回报率读回不匹配；原值已恢复并读回".into(),
                    Err(error) => {
                        format!("Viper V4 Pro 回报率读回不匹配；原值恢复也未确认：{error}")
                    }
                });
            }
            Ok(())
        }
        Action::Sleep(seconds) => {
            if !SLEEP.contains(&seconds) {
                return Err("Viper V4 Pro 休眠时间仅支持 60、300、600 或 900 秒".into());
            }
            let before = exchange(device, 0x07, 0x83, 2, &[])?;
            if before.len() < 2 {
                return Err("Viper V4 Pro 原休眠设置字段缺失".into());
            }
            let old = [before[0], before[1]];
            if let Err(error) = exchange(device, 0x07, 0x03, 2, &seconds.to_be_bytes()) {
                return match restore_sleep(device, old) {
                    Ok(()) => Err(format!("休眠设置写入未确认：{error}；原值已恢复并读回")),
                    Err(restore) => Err(format!(
                        "休眠设置写入未确认：{error}；原值恢复也未确认：{restore}"
                    )),
                };
            }
            let actual = match exchange(device, 0x07, 0x83, 2, &[]) {
                Ok(actual) => actual,
                Err(error) => {
                    let rollback = restore_sleep(device, old);
                    return Err(match rollback {
                        Ok(()) => format!("休眠设置读回失败：{error}；原值已恢复并读回"),
                        Err(restore) => {
                            format!("休眠设置读回失败：{error}；原值恢复也未确认：{restore}")
                        }
                    });
                }
            };
            if actual.as_slice() != seconds.to_be_bytes() {
                let rollback = restore_sleep(device, old);
                return Err(match rollback {
                    Ok(()) => "Viper V4 Pro 休眠时间读回不匹配；原值已恢复并读回".into(),
                    Err(error) => {
                        format!("Viper V4 Pro 休眠时间读回不匹配；原值恢复也未确认：{error}")
                    }
                });
            }
            Ok(())
        }
        _ => Err("Viper V4 Pro 插件仅提供电量、DPI、回报率与休眠设置".into()),
    }
}

fn restore_rate(device: &Device, old: u8) -> Result<(), String> {
    exchange(device, 0x00, 0x40, 2, &[1, old])?;
    thread::sleep(Duration::from_millis(150));
    let actual = exchange(device, 0x00, 0xc0, 2, &[1])?;
    if actual.get(1) == Some(&old) {
        Ok(())
    } else {
        Err("原回报率恢复读回不匹配".into())
    }
}

fn restore_sleep(device: &Device, old: [u8; 2]) -> Result<(), String> {
    exchange(device, 0x07, 0x03, 2, &old)?;
    if exchange(device, 0x07, 0x83, 2, &[])?.as_slice() == old {
        Ok(())
    } else {
        Err("原休眠设置恢复读回不匹配".into())
    }
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if key != DEVICE_KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let request: Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        match request.get("op").and_then(Value::as_str) {
            Some("read-state") => {
                let (device, pid) = open_unique()?;
                serde_json::to_value(read_state(&device, pid)?).map_err(|e| e.to_string())
            }
            Some("apply-setting") => {
                let action: Action =
                    serde_json::from_value(request.get("action").cloned().ok_or("缺少设置操作")?)
                        .map_err(|e| format!("设置操作无效：{e}"))?;
                let (device, _) = open_unique()?;
                apply(&device, action)?;
                Ok(Value::Null)
            }
            Some("controls-read" | "controls-apply") => {
                Err("Viper V4 Pro 未提供已确认的改键或宏接口".into())
            }
            _ => Err("Viper V4 Pro 支持电量、DPI、回报率与休眠设置".into()),
        }
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("razer-viper-v4-pro", handler);
