//! ATK F1 Pro Max, scoped to its documented 17-byte report-08 collections.
//! Battery framing: Vigil Watt v1.3.0. DPI/rate EEPROM operations are adapted
//! from OpenMouse mouse-protocol commit 23f5bed75d148cabeed19c74838deb31d3f60d02.
use peripheral_desk::{
    hid::{self, Interface},
    model::{Action, State},
};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "atk-f1-pro-max";
const VID: u16 = 0x373b;
const RECEIVER_1K: u16 = 0x101a;
const RECEIVER_8K: u16 = 0x101b;
const WIRED: u16 = 0x1015;
const REPORT_ID: u8 = 0x08;
const REPORT_LEN: usize = 17;
const PAYLOAD_LEN: usize = 16;
const USAGE_PAGE: u16 = 0xff02;
const USAGE: u16 = 0x0002;
const CID: u8 = 2;
const MID: u8 = 37;
const TIMEOUT: Duration = Duration::from_millis(700);

fn matches(i: &Interface) -> bool {
    let pid_ok = match i.pid {
        RECEIVER_1K | RECEIVER_8K => {
            i.input_len as usize == REPORT_LEN && i.output_len as usize == REPORT_LEN
        }
        WIRED => i.input_len as usize >= REPORT_LEN && i.output_len as usize >= REPORT_LEN,
        _ => false,
    };
    i.vid == VID
        && pid_ok
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.path.to_ascii_lowercase().contains("mi_01")
}

fn select() -> Result<Interface, String> {
    let found: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
    match found.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err("未找到 F1 Pro Max 精确 report-08 电量/设置接口（373B:101A/101B 接收器或 1015 本体，FF02:0002，MI_01）".into()),
        _ => Err("检测到多个 F1 Pro Max 协议接口；为避免写错设备，已停止".into()),
    }
}

fn checksum(payload: &[u8; PAYLOAD_LEN]) -> u8 {
    let sum = payload[..15]
        .iter()
        .fold(REPORT_ID as u16, |s, b| s + u16::from(*b));
    0x55u16.wrapping_sub(sum & 0xff) as u8
}

fn valid_reply(bytes: &[u8]) -> bool {
    bytes.len() == PAYLOAD_LEN
        && bytes
            .iter()
            .fold(REPORT_ID as u16, |s, b| s + u16::from(*b))
            & 0xff
            == 0x55
}

fn packet(command: u8) -> [u8; PAYLOAD_LEN] {
    let mut p = [0; PAYLOAD_LEN];
    p[0] = command;
    p[15] = checksum(&p);
    p
}

fn exchange(
    transport: &mut WinTransport,
    req: &[u8; PAYLOAD_LEN],
) -> Result<[u8; PAYLOAD_LEN], String> {
    transport.flush()?;
    let mut wire = [0; REPORT_LEN];
    wire[0] = REPORT_ID;
    wire[1..].copy_from_slice(req);
    transport.write(&wire, Duration::from_millis(200))?;
    let deadline = Instant::now() + TIMEOUT;
    while Instant::now() < deadline {
        let reply = transport.read(
            REPORT_LEN,
            deadline.saturating_duration_since(Instant::now()),
        )?;
        if reply.is_empty() {
            break;
        }
        if reply.len() != REPORT_LEN || reply[0] != REPORT_ID {
            continue;
        }
        let payload: [u8; PAYLOAD_LEN] = reply[1..].try_into().map_err(|_| "ATK 回复长度错误")?;
        if payload[0] != req[0] {
            continue;
        }
        if !valid_reply(&payload) {
            return Err("ATK report-08 回复校验和不匹配".into());
        }
        if payload[1] != 0 {
            return Err(format!("ATK 命令状态错误 0x{:02X}", payload[1]));
        }
        return Ok(payload);
    }
    Err(format!(
        "ATK 未在 {} ms 内回复命令 0x{:02X}",
        TIMEOUT.as_millis(),
        req[0]
    ))
}

fn request(command_id: u8, address: u16, len: u8, data: &[u8]) -> [u8; PAYLOAD_LEN] {
    let mut p = packet(command_id);
    p[1] = 0;
    p[2] = (address >> 8) as u8;
    p[3] = address as u8;
    p[4] = len;
    p[5..5 + data.len()].copy_from_slice(data);
    p[15] = checksum(&p);
    p
}

fn exchange_cmd(t: &mut WinTransport, command_id: u8, data: &[u8]) -> Result<Vec<u8>, String> {
    let mut p = packet(command_id);
    p[4] = data.len() as u8;
    p[5..5 + data.len()].copy_from_slice(data);
    p[15] = checksum(&p);
    let reply = exchange(t, &p)?;
    if reply[4] != data.len() as u8 {
        return Err("ATK 回复数据长度不匹配".into());
    }
    Ok(reply[5..5 + data.len()].to_vec())
}

fn read_eeprom(t: &mut WinTransport, address: u16, len: u8) -> Result<Vec<u8>, String> {
    let req = request(0x08, address, len, &[]);
    let reply = exchange(t, &req)?;
    if reply[4] != len || reply[2] != (address >> 8) as u8 || reply[3] != address as u8 {
        return Err("ATK EEPROM 读取回复地址或长度不匹配".into());
    }
    Ok(reply[5..5 + len as usize].to_vec())
}

fn write_eeprom(t: &mut WinTransport, address: u16, data: &[u8]) -> Result<(), String> {
    if data.is_empty() || data.len() > 10 {
        return Err("ATK EEPROM 写入长度无效".into());
    }
    let req = request(0x07, address, data.len() as u8, data);
    let reply = exchange(t, &req)?;
    if reply[4] != data.len() as u8 || reply[2] != (address >> 8) as u8 || reply[3] != address as u8
    {
        return Err("ATK EEPROM 写入应答地址或长度不匹配".into());
    }
    Ok(())
}

fn transaction(
    t: &mut WinTransport,
    address: u16,
    before: &[u8],
    after: &[u8],
) -> Result<(), String> {
    let applied = write_eeprom(t, address, after).and_then(|_| {
        let got = read_eeprom(t, address, before.len() as u8)?;
        if got == after {
            Ok(())
        } else {
            Err("EEPROM 写入读回不一致".into())
        }
    });
    if let Err(e) = applied {
        let restored = write_eeprom(t, address, before)
            .and_then(|_| read_eeprom(t, address, before.len() as u8))
            .and_then(|got| {
                if got == before {
                    Ok(())
                } else {
                    Err("原值恢复后读回不一致".into())
                }
            });
        return Err(match restored {
            Ok(()) => format!("写入失败：{e}；原值已恢复并核对"),
            Err(r) => format!("写入失败：{e}；原值恢复未确认：{r}"),
        });
    }
    Ok(())
}

fn check_identity(t: &mut WinTransport) -> Result<(), String> {
    let id = exchange_cmd(t, 0x10, &[0, 0, 0])?;
    if id.get(0..2) == Some(&[CID, MID]) {
        Ok(())
    } else {
        Err(format!(
            "ATK CID/MID 不匹配：{}；拒绝读取或写入 F1 Pro Max 设置",
            id.first().copied().unwrap_or(255)
        ))
    }
}

fn unpack_dpi(b: &[u8]) -> Result<u16, String> {
    if b.len() < 4
        || b[0]
            .wrapping_add(b[1])
            .wrapping_add(b[2])
            .wrapping_add(b[3])
            != 0x55
    {
        return Err("ATK DPI stage 校验失败".into());
    }
    let count = u16::from(b[0]) | (u16::from((b[2] & 0x0c) >> 2) << 8);
    let value = (count + 1) * 50;
    Ok(if b[2] & 1 != 0 { value * 2 } else { value })
}

fn pack_dpi(dpi: u16) -> [u8; 4] {
    let doubled = dpi > 30_000;
    let count = dpi / if doubled { 100 } else { 50 } - 1;
    let lo = count as u8;
    let nibble = (((count >> 8) as u8 & 3) << 2) | u8::from(doubled);
    let mode = nibble | (nibble << 4);
    [
        lo,
        lo,
        mode,
        0x55u8.wrapping_sub(lo.wrapping_mul(2).wrapping_add(mode)),
    ]
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn read_state() -> Result<State, String> {
    let interface = select()?;
    let wired = interface.pid == WIRED;
    let mut t = WinTransport::open(&interface.path)?;
    let identity = exchange_cmd(&mut t, 0x10, &[0, 0, 0])?;
    if identity.get(0..2) != Some(&[CID, MID]) {
        return Err("ATK CID/MID 不是 F1 Pro Max 2,37；未查询电量或设置".into());
    }
    let battery = exchange_cmd(&mut t, 0x04, &[0; 2])?;
    if battery.len() != 2 || battery[0] > 100 || battery[1] > 1 {
        return Err("F1 Pro Max 电量或充电字段超出协议范围".into());
    }
    let system = read_eeprom(&mut t, 0, 6)?;
    if system.len() != 6
        || system[0].wrapping_add(system[1]) != 0x55
        || system[2].wrapping_add(system[3]) != 0x55
        || system[4].wrapping_add(system[5]) != 0x55
        || system[2] == 0
        || system[2] > 6
        || system[4] >= system[2]
    {
        return Err("F1 Pro Max DPI 档位配置校验失败；保持设置未知".into());
    }
    let rates = [
        (0x08, 125),
        (0x04, 250),
        (0x02, 500),
        (0x01, 1000),
        (0x10, 2000),
        (0x20, 4000),
        (0x40, 8000),
    ];
    let rate = rates
        .iter()
        .find(|(c, hz)| *c == system[0] && (!wired || *hz <= 1000))
        .map(|(_, hz)| *hz);
    let active = system[4] as usize;
    let dpi_bytes = read_eeprom(&mut t, 0x000c + active as u16 * 4, 4)?;
    let dpi = unpack_dpi(&dpi_bytes)?;
    if !(50..=36_000).contains(&dpi)
        || (dpi <= 30_000 && !dpi.is_multiple_of(50))
        || (dpi > 30_000 && !dpi.is_multiple_of(100))
    {
        return Err("F1 Pro Max DPI 超出 PAW3950 范围".into());
    }
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        wired,
        percent: Some(battery[0]),
        charging: Some(battery[1] == 1),
        dpi: Some(dpi),
        rate,
        updated: now(),
        identity: format!("ATK F1 Pro Max · 373B:{:04X} · CID/MID 2,37", interface.pid),
        settings_error: if rate.is_some() {
            String::new()
        } else {
            "当前连接返回了不支持的回报率编码".into()
        },
        error: String::new(),
        ..Default::default()
    })
}

fn apply(action: Action) -> Result<(), String> {
    let interface = select()?;
    let wired = interface.pid == WIRED;
    let mut t = WinTransport::open(&interface.path)?;
    check_identity(&mut t)?;
    match action {
        Action::Dpi(dpi) => {
            if !(50..=36_000).contains(&dpi)
                || (dpi <= 30_000 && !dpi.is_multiple_of(50))
                || (dpi > 30_000 && !dpi.is_multiple_of(100))
            {
                return Err(
                    "PAW3950 DPI 支持 50–30000（每 50 递增）或 30100–36000（每 100 递增）".into(),
                );
            }
            let system = read_eeprom(&mut t, 0, 6)?;
            if system[2].wrapping_add(system[3]) != 0x55
                || system[4].wrapping_add(system[5]) != 0x55
                || system[2] == 0
                || system[2] > 6
                || system[4] >= system[2]
            {
                return Err("活动 DPI 档位字段校验失败；拒绝写入".into());
            }
            let addr = 0x000c + u16::from(system[4]) * 4;
            let before = read_eeprom(&mut t, addr, 4)?;
            transaction(&mut t, addr, &before, &pack_dpi(dpi))
        }
        Action::Rate(hz) => {
            let code = match hz {
                125 => 0x08,
                250 => 0x04,
                500 => 0x02,
                1000 => 0x01,
                2000 => 0x10,
                4000 => 0x20,
                8000 => 0x40,
                _ => return Err("回报率支持 125/250/500/1000/2000/4000/8000 Hz".into()),
            };
            let max = if wired || interface.pid == RECEIVER_1K {
                1000
            } else {
                8000
            };
            if hz > max {
                return Err(format!("当前连接最高回报率为 {max} Hz"));
            }
            let before = read_eeprom(&mut t, 0, 2)?;
            if before.len() != 2 || before[0].wrapping_add(before[1]) != 0x55 {
                return Err("原回报率字段校验失败；拒绝写入".into());
            }
            transaction(&mut t, 0, &before, &[code, 0x55u8.wrapping_sub(code)])
        }
        _ => Err("ATK F1 Pro Max 此插件支持电量、DPI 和回报率；改键/宏/灯效未适配".into()),
    }
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        if key != KEY {
            return Err("驱动型号标识不匹配".into());
        }
        let req: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match req.get("op").and_then(Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            Some("apply-setting") => {
                let action: Action =
                    serde_json::from_value(req.get("action").cloned().ok_or("缺少设置操作")?)
                        .map_err(|e| e.to_string())?;
                apply(action)?;
                Ok(Value::Null)
            }
            _ => Err("ATK F1 Pro Max 支持状态读取及 DPI/回报率设置".into()),
        }
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}).to_string(),
        Err(error) => json!({"ok":false,"value":null,"error":error}).to_string(),
    }
}

peripheral_desk_driver_ffi::export_driver!("atk-f1-pro-max", handler);
