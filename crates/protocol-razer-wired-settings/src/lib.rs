//! Read and write DPI and polling rate for exact, upstream-identified Razer mice.
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::{Action, State},
};
use std::time::Duration;

const VID: u16 = 0x1532;
const REPORT_LEN: usize = 91;

#[derive(Clone, Copy)]
pub enum RateProtocol {
    Standard { transaction_id: u8 },
    Viper8k,
}

#[derive(Clone, Copy)]
pub struct Profile {
    pub key: &'static str,
    pub model: &'static str,
    pub pid: u16,
    pub dpi_max: u16,
    pub dpi_transaction_id: u8,
    pub rate: RateProtocol,
}

fn checksum(report: &[u8; REPORT_LEN]) -> u8 {
    report[3..89].iter().fold(0, |sum, byte| sum ^ byte)
}

fn request(
    tid: u8,
    class: u8,
    command: u8,
    size: u8,
    args: &[u8],
) -> Result<[u8; REPORT_LEN], String> {
    if args.len() > size as usize || 9 + size as usize > 89 {
        return Err("Razer 请求参数长度无效".into());
    }
    let mut wire = [0; REPORT_LEN];
    wire[2] = tid;
    wire[6] = size;
    wire[7] = class;
    wire[8] = command;
    wire[9..9 + args.len()].copy_from_slice(args);
    wire[89] = checksum(&wire);
    Ok(wire)
}

fn reply_body<'a>(wire: &'a [u8], req: &[u8; REPORT_LEN]) -> Result<&'a [u8], String> {
    let wire = <&[u8; REPORT_LEN]>::try_from(wire)
        .map_err(|_| "Razer Feature 回复长度不匹配".to_string())?;
    if wire[0] != 0
        || wire[89] != checksum(wire)
        || wire[2] != req[2]
        || wire[3..5] != req[3..5]
        || wire[5] != 0
        || wire[6] != req[6]
        || wire[7] != req[7]
        || wire[8] != req[8]
    {
        return Err("Razer Feature 回复校验或命令匹配失败".into());
    }
    match wire[1] {
        0x02 => Ok(&wire[9..9 + wire[6] as usize]),
        0x01 => Err("Razer 设备仍在处理请求".into()),
        status => Err(format!("Razer 设备未完成请求（状态 0x{status:02X}）")),
    }
}

fn transact(device: &Device, req: &[u8; REPORT_LEN], wait: Duration) -> Result<Vec<u8>, String> {
    device.send(req)?;
    std::thread::sleep(wait);
    reply_body(&device.receive(0, REPORT_LEN)?, req).map(|body| body.to_vec())
}

fn query(
    device: &Device,
    tid: u8,
    class: u8,
    command: u8,
    size: u8,
    args: &[u8],
    wait: Duration,
) -> Result<Vec<u8>, String> {
    let req = request(tid, class, command, size, args)?;
    let mut last = String::new();
    for attempt in 0..5 {
        match transact(device, &req, wait) {
            Ok(body) => return Ok(body),
            Err(error) => last = error,
        }
        if attempt < 4 {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Err(last)
}

fn write(
    device: &Device,
    tid: u8,
    class: u8,
    command: u8,
    size: u8,
    args: &[u8],
    wait: Duration,
) -> Result<(), String> {
    let req = request(tid, class, command, size, args)?;
    transact(device, &req, wait).map(|_| ())
}

fn matching(i: &Interface, profile: Profile) -> bool {
    i.vid == VID
        && i.pid == profile.pid
        && i.feature_len == REPORT_LEN as u16
        && i.path.to_ascii_lowercase().contains("mi_00")
}

fn unique(profile: Profile) -> Result<Option<Device>, String> {
    let interfaces = hid::enumerate()?;
    let matches: Vec<_> = interfaces.iter().filter(|i| matching(i, profile)).collect();
    match matches.as_slice() {
        [] => Ok(None),
        [interface] => Device::open(&interface.path).map(Some),
        _ => Err(format!(
            "发现多个 Razer {} 匹配接口，已停止操作",
            profile.model
        )),
    }
}

fn wait() -> Duration {
    Duration::from_micros(600)
}

fn read_dpi_xy(device: &Device, p: Profile) -> Result<(u16, u16), String> {
    let body = query(device, p.dpi_transaction_id, 0x04, 0x85, 7, &[0], wait())?;
    if body.len() != 7 {
        return Err("Razer DPI 回复字段长度无效".into());
    }
    let x = u16::from_be_bytes([body[1], body[2]]);
    let y = u16::from_be_bytes([body[3], body[4]]);
    if !(100..=p.dpi_max).contains(&x) || !(100..=p.dpi_max).contains(&y) {
        return Err("Razer DPI 回复超出型号范围".into());
    }
    Ok((x, y))
}

fn rate_from_code(code: u8) -> Result<u16, String> {
    match code {
        1 => Ok(1000),
        2 => Ok(500),
        8 => Ok(125),
        _ => Err(format!("Razer 未知回报率字段 0x{code:02X}")),
    }
}

fn read_viper_8k_rate(device: &Device) -> Result<u16, String> {
    let req = request(0x1f, 0x00, 0xc0, 1, &[])?;
    let mut last = String::new();
    for attempt in 0..5 {
        device.send(&req)?;
        std::thread::sleep(wait());
        let reply = device.receive(0, REPORT_LEN)?;
        match reply_body(&reply, &req) {
            Ok(_) => match reply.get(10).copied() {
                Some(0x01) => return Ok(8000),
                Some(0x02) => return Ok(4000),
                Some(0x04) => return Ok(2000),
                Some(0x08) => return Ok(1000),
                Some(0x10) => return Ok(500),
                Some(0x40) => return Ok(125),
                Some(v) => return Err(format!("Razer 未知回报率字段 0x{v:02X}")),
                None => return Err("Razer 回报率回复字段缺失".into()),
            },
            Err(error) => last = error,
        }
        if attempt < 4 {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Err(last)
}

fn read_rate(device: &Device, p: Profile) -> Result<u16, String> {
    if matches!(p.rate, RateProtocol::Viper8k) {
        return read_viper_8k_rate(device);
    }
    let (tid, class, command, size, rates) = match p.rate {
        RateProtocol::Standard { transaction_id } => {
            (transaction_id, 0x00, 0x85, 1, &[125, 500, 1000][..])
        }
        RateProtocol::Viper8k => unreachable!(),
    };
    let body = query(device, tid, class, command, size, &[], wait())?;
    let value = match p.rate {
        RateProtocol::Standard { .. } => {
            rate_from_code(*body.first().ok_or("Razer 回报率回复字段缺失")?)?
        }
        RateProtocol::Viper8k => unreachable!(),
    };
    if rates.contains(&value) {
        Ok(value)
    } else {
        Err("Razer 回报率不在型号支持档位内".into())
    }
}

fn read_state_from(device: &Device, p: Profile) -> State {
    let mut s = State {
        key: p.key.into(),
        present: true,
        wired: true,
        identity: format!("Razer {} · USB 1532:{:04X}", p.model, p.pid),
        ..State::default()
    };
    match read_dpi_xy(device, p) {
        Ok((x, _)) => {
            s.dpi = Some(x);
            s.fresh = true;
        }
        Err(e) => s.settings_error = format!("DPI：{e}"),
    }
    match read_rate(device, p) {
        Ok(v) => {
            s.rate = Some(v);
            s.fresh = true;
        }
        Err(e) => {
            if !s.settings_error.is_empty() {
                s.settings_error.push(' ');
            }
            s.settings_error.push_str(&format!("回报率：{e}"));
        }
    }
    s
}

pub fn read_state(p: Profile) -> Result<State, String> {
    match unique(p)? {
        Some(device) => Ok(read_state_from(&device, p)),
        None => Ok(State {
            key: p.key.into(),
            wired: true,
            identity: format!("Razer {} · USB 1532:{:04X}", p.model, p.pid),
            error: format!("未找到唯一的 Razer {} 有线 Feature 接口", p.model),
            ..State::default()
        }),
    }
}

pub fn apply_setting(p: Profile, action: Action) -> Result<(), String> {
    let device = unique(p)?.ok_or("设备尚未连接")?;
    let wait = wait();
    match action {
        Action::Dpi(dpi) if (100..=p.dpi_max).contains(&dpi) && dpi.is_multiple_of(50) => {
            let old = read_dpi_xy(&device, p)?;
            let args = [
                0,
                (dpi >> 8) as u8,
                dpi as u8,
                (dpi >> 8) as u8,
                dpi as u8,
                0,
                0,
            ];
            write(&device, p.dpi_transaction_id, 0x04, 0x05, 7, &args, wait)?;
            if read_dpi_xy(&device, p)? != (dpi, dpi) {
                let restore = [
                    0,
                    (old.0 >> 8) as u8,
                    old.0 as u8,
                    (old.1 >> 8) as u8,
                    old.1 as u8,
                    0,
                    0,
                ];
                write(&device, p.dpi_transaction_id, 0x04, 0x05, 7, &restore, wait)?;
                return Err("DPI 读回与目标值不匹配；原值已恢复".into());
            }
            Ok(())
        }
        Action::Rate(rate) => set_rate(&device, p, rate),
        Action::Dpi(_) => Err(format!("DPI 必须为 100–{}，且是 50 的倍数", p.dpi_max)),
        _ => Err("此型号只开放 DPI 与回报率设置".into()),
    }
}

fn set_rate(device: &Device, p: Profile, rate: u16) -> Result<(), String> {
    let supported = match p.rate {
        RateProtocol::Standard { .. } => [125, 500, 1000].contains(&rate),
        RateProtocol::Viper8k => [125, 500, 1000, 2000, 4000, 8000].contains(&rate),
    };
    if !supported {
        return Err("回报率不在此型号支持档位内".into());
    }
    let old = read_rate(device, p)?;
    let updated = write_rate_value(device, p, rate);
    if let Err(error) = updated {
        return match write_rate_value(device, p, old) {
            Ok(()) => Err(format!("回报率设置未确认：{error}；已重新发送原值")),
            Err(restore) => Err(format!(
                "回报率设置未确认：{error}；恢复原值也未确认：{restore}"
            )),
        };
    }
    match read_rate(device, p) {
        Ok(actual) if actual == rate => Ok(()),
        readback => {
            let cause = match readback {
                Ok(actual) => format!("读回不匹配，期望 {rate} Hz，实际 {actual} Hz"),
                Err(error) => format!("读回失败：{error}"),
            };
            match write_rate_value(device, p, old) {
                Ok(()) => Err(format!("回报率{cause}；已重新发送原值")),
                Err(error) => Err(format!("回报率{cause}；恢复原值未确认：{error}")),
            }
        }
    }
}

fn write_rate_value(device: &Device, p: Profile, rate: u16) -> Result<(), String> {
    match p.rate {
        RateProtocol::Standard { transaction_id } => {
            let code = match rate {
                1000 => 1,
                500 => 2,
                125 => 8,
                _ => return Err("不支持的回报率".into()),
            };
            write(device, transaction_id, 0, 0x05, 1, &[code], wait())
        }
        RateProtocol::Viper8k => {
            let code = match rate {
                8000 => 1,
                4000 => 2,
                2000 => 4,
                1000 => 8,
                500 => 0x10,
                125 => 0x40,
                _ => return Err("不支持的回报率".into()),
            };
            write(device, 0x1f, 0, 0x40, 2, &[0, code], wait())?;
            write(device, 0xff, 0, 0x40, 2, &[1, code], wait())
        }
    }
}

pub fn dispatch(key: &str, input: &str, p: Profile) -> String {
    let result = (|| {
        if key != p.key {
            return Err("驱动型号标识不匹配".into());
        }
        let req: serde_json::Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        match req.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state(p)?).map_err(|e| e.to_string()),
            Some("apply-setting") => {
                let action = req.get("action").ok_or("缺少驱动参数：action")?;
                let action: Action = serde_json::from_value(action.clone())
                    .map_err(|e| format!("驱动参数 action 无效：{e}"))?;
                apply_setting(p, action)?;
                Ok(serde_json::Value::Null)
            }
            Some(op) => Err(format!("{} 不支持操作：{op}", p.model)),
            None => Err("缺少驱动操作类型".into()),
        }
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
