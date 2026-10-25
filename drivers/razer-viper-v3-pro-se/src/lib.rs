//! Razer Viper V3 Pro SE receiver adapter.
//! Battery response evidence is Vigil Watt v1.3.0; DPI/rate settings follow
//! OpenMouse's model-specific SE profile (AGPL-3.0-or-later).
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::Action,
};
use serde_json::{json, Value};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const KEY: &str = "razer-viper-v3-pro-se";
const VID: u16 = 0x1532;
const PID: u16 = 0x00df;
const LEN: usize = 91;
const TID: u8 = 0x1f;
const RATES: [(u16, u8); 3] = [(125, 8), (500, 2), (1000, 1)];

fn matches(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.feature_len == LEN as u16
        && i.usage_page == 0x0001
        && i.usage == 0x0002
        && i.path.to_ascii_lowercase().contains("mi_00")
}
fn open() -> Result<Device, String> {
    let ports: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    match ports.as_slice() {
        [i] => Device::open(&i.path),
        [] => Err("未找到 Viper V3 Pro SE 无线接收器的 91 字节 Feature 通道".into()),
        _ => Err("检测到多个 Viper V3 Pro SE 接口；已停止操作".into()),
    }
}
fn xor(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |a, b| a ^ b)
}
fn request(class: u8, command: u8, size: u8, args: &[u8]) -> Result<[u8; LEN], String> {
    if size > 80 || args.len() > size as usize {
        return Err("Razer 请求长度无效".into());
    }
    let mut w = [0u8; LEN];
    w[2] = TID;
    w[6] = size;
    w[7] = class;
    w[8] = command;
    w[9..9 + args.len()].copy_from_slice(args);
    w[89] = xor(&w[3..89]);
    Ok(w)
}
fn query(d: &Device, c: u8, id: u8, size: u8, args: &[u8]) -> Result<Vec<u8>, String> {
    let req = request(c, id, size, args)?;
    for attempt in 0..6 {
        d.send(&req)?;
        thread::sleep(Duration::from_millis(60));
        let r = d.receive(0, LEN)?;
        if r.len() != LEN || r[0] != 0 || r[89] != xor(&r[3..89]) {
            return Err("Razer Feature 回复长度、Report ID 或校验不匹配".into());
        }
        match r[1] {
            0x01 if attempt < 5 => continue,
            0x04 => return Err("设备未响应，可能休眠".into()),
            0x02 => {}
            s => return Err(format!("设备回复状态 0x{s:02X}")),
        }
        if r[2] != TID || r[7] != c || r[8] != id || r[5] != 0 || r[6] != size {
            return Err("Razer 回复不属于本次请求".into());
        }
        return Ok(r[9..9 + size as usize].to_vec());
    }
    Err("Razer 设备持续忙".into())
}
fn dpi(d: &Device) -> Result<(u16, u16), String> {
    let v = query(d, 0x04, 0x85, 7, &[1])?;
    if v.len() != 7 {
        return Err("DPI 回复字段长度异常".into());
    }
    let x = u16::from_be_bytes([v[1], v[2]]);
    let y = u16::from_be_bytes([v[3], v[4]]);
    if !(100..=35_000).contains(&x) || !(100..=35_000).contains(&y) {
        return Err("DPI 超出型号范围".into());
    }
    Ok((x, y))
}
fn rate(d: &Device) -> Result<(u16, u8), String> {
    let v = query(d, 0, 0x85, 1, &[])?;
    let code = *v.first().ok_or("回报率字段缺失")?;
    let hz = RATES
        .iter()
        .find(|(_, c)| *c == code)
        .map(|(h, _)| *h)
        .ok_or_else(|| format!("无线接收器回报率字段未知 0x{code:02X}"))?;
    Ok((hz, code))
}
fn percent(raw: u8) -> u8 {
    if raw <= 100 {
        raw
    } else {
        (u16::from(raw) * 100 / 255) as u8
    }
}
fn read_state(d: &Device) -> Result<Value, String> {
    let battery = query(d, 7, 0x80, 2, &[0, 0])?;
    let charge = query(d, 7, 0x84, 2, &[0, 0]);
    let (x, y) = dpi(d)?;
    let (hz, _) = rate(d)?;
    if battery.len() != 2 {
        return Err("电量回复字段缺失".into());
    }
    let charge = charge.ok().and_then(|v| match v.get(1) {
        Some(0) => Some(false),
        Some(1) => Some(true),
        _ => None,
    });
    let raw = battery[1];
    Ok(
        json!({"key":KEY,"present":true,"fresh":true,"percent":percent(raw),"battery_raw":raw,
      "connection_path":"2.4 GHz 接收器","charging":charge,"wired":false,"dpi":x,"rate":hz,
      "updated":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
      "identity":"Razer Viper V3 Pro SE · 1532:00DF · MI_00 · 91-byte Feature","error":"","settings_error":"改键、宏和灯效无此型号的已确认实现。","tuning":{},"dpi_stages":[],"active_stage":null,"dpi_y":y}),
    )
}
fn rollback(
    d: &Device,
    class: u8,
    write: u8,
    read: u8,
    size: u8,
    old: &[u8],
) -> Result<(), String> {
    query(d, class, write, size, old)?;
    let current = query(d, class, read, size, &[])?;
    if current == old {
        Ok(())
    } else {
        Err("原设置恢复后的读回不匹配".into())
    }
}
fn apply(d: &Device, a: Action) -> Result<(), String> {
    match a {
        Action::Dpi(v) => {
            if !(100..=35_000).contains(&v) || !v.is_multiple_of(50) {
                return Err("DPI 必须在 100–35000 之间，按 50 递增".into());
            }
            let old = query(d, 4, 0x85, 7, &[1])?;
            if old.len() != 7 {
                return Err("原 DPI 字段缺失".into());
            }
            let mut args = old.clone();
            args[0] = 1;
            args[1..5].copy_from_slice(&[
                v.to_be_bytes()[0],
                v.to_be_bytes()[1],
                v.to_be_bytes()[0],
                v.to_be_bytes()[1],
            ]);
            if let Err(e) = query(d, 4, 0x05, 7, &args) {
                let rb = rollback(d, 4, 0x05, 0x85, 7, &old);
                return Err(format!("DPI 写入未确认：{e}；恢复结果：{rb:?}"));
            }
            let verify = dpi(d);
            if !matches!(verify,Ok((x,y)) if x==v&&y==v) {
                let rb = rollback(d, 4, 0x05, 0x85, 7, &old);
                return Err(format!("DPI 读回不符；恢复结果：{rb:?}"));
            }
            Ok(())
        }
        Action::Rate(hz) => {
            let code = RATES
                .iter()
                .find(|(r, _)| *r == hz)
                .map(|(_, c)| *c)
                .ok_or("该接收器仅支持 125、500、1000 Hz")?;
            let (_, old) = rate(d)?;
            let original = [old];
            if let Err(e) = query(d, 0, 0x05, 1, &[code]) {
                let rb = rollback(d, 0, 0x05, 0x85, 1, &original);
                return Err(format!("回报率写入未确认：{e}；恢复结果：{rb:?}"));
            }
            if rate(d)?.0 != hz {
                let rb = rollback(d, 0, 0x05, 0x85, 1, &original);
                return Err(format!("回报率读回不符；恢复结果：{rb:?}"));
            }
            Ok(())
        }
        _ => {
            Err("Viper V3 Pro SE 预览仅支持电量、DPI 和回报率；休眠设置尚未由本型号来源确认".into())
        }
    }
}
fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if key != KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let req: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match req.get("op").and_then(Value::as_str) {
            Some("read-state") => read_state(&open()?),
            Some("apply-setting") => {
                let a: Action =
                    serde_json::from_value(req.get("action").cloned().ok_or("缺少设置操作")?)
                        .map_err(|e| e.to_string())?;
                apply(&open()?, a)?;
                Ok(Value::Null)
            }
            _ => Err("Viper V3 Pro SE 预览仅支持电量、DPI 和回报率".into()),
        }
    })();
    match result {
        Ok(v) => json!({"ok":true,"value":v,"error":""}),
        Err(e) => json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("razer-viper-v3-pro-se", handler);
