//! Razer Viper V2 Pro wired settings for one explicitly matched USB interface.
//!
//! Protocol facts are derived from OpenRazer at commit
//! `477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`; see docs/RAZER-VIPER-V2-PRO.md.
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::{Action, State},
};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const DEVICE_KEY: &str = "razer-viper-v2-pro-wired";
const VID: u16 = 0x1532;
const PID: u16 = 0x00a5;
const REPORT_LEN: usize = 91;
const TRANSACTION_ID: u8 = 0x1f;
const RESPONSE_WAIT: Duration = Duration::from_millis(31);

const BATTERY: (u8, u8, u8) = (0x07, 0x80, 2);
const CHARGING: (u8, u8, u8) = (0x07, 0x84, 2);
const DPI_READ: (u8, u8, u8) = (0x04, 0x85, 7);
const DPI_STAGES_READ: (u8, u8, u8) = (0x04, 0x86, 38);
const DPI_STAGES_WRITE: (u8, u8, u8) = (0x04, 0x06, 38);
const DPI_WRITE: (u8, u8, u8) = (0x04, 0x05, 7);
const RATE_READ: (u8, u8, u8) = (0x00, 0x85, 1);
const RATE_WRITE: (u8, u8, u8) = (0x00, 0x05, 1);

fn matching_interface(interface: &Interface) -> bool {
    interface.vid == VID
        && interface.pid == PID
        && interface.feature_len == REPORT_LEN as u16
        && interface.path.to_ascii_lowercase().contains("mi_00")
}

fn checksum(wire: &[u8; REPORT_LEN]) -> u8 {
    wire[3..89].iter().fold(0, |sum, byte| sum ^ byte)
}

fn request(class: u8, command: u8, size: u8, args: &[u8]) -> Result<[u8; REPORT_LEN], String> {
    if args.len() > size as usize || 9 + size as usize > 89 {
        return Err("Razer 请求参数长度无效".into());
    }
    let mut wire = [0; REPORT_LEN];
    wire[2] = TRANSACTION_ID;
    wire[6] = size;
    wire[7] = class;
    wire[8] = command;
    wire[9..9 + args.len()].copy_from_slice(args);
    wire[89] = checksum(&wire);
    Ok(wire)
}

fn reply_body<'a>(wire: &'a [u8], request: &[u8; REPORT_LEN]) -> Result<&'a [u8], String> {
    let Ok(wire) = <&[u8; REPORT_LEN]>::try_from(wire) else {
        return Err("Razer Feature 回复长度不匹配".into());
    };
    if wire[0] != 0 || wire[89] != checksum(wire) {
        return Err("Razer Feature 回复的 Report ID 或校验不匹配".into());
    }
    if wire[2] != request[2]
        || wire[3..5] != request[3..5]
        || wire[5] != 0
        || wire[6] != request[6]
        || wire[7] != request[7]
        || wire[8] != request[8]
    {
        return Err("Razer Feature 回复不属于本次查询或设置".into());
    }
    match wire[1] {
        0x02 => Ok(&wire[9..9 + wire[6] as usize]),
        0x01 => Err("Razer 设备仍在处理请求".into()),
        status => Err(format!("Razer 设备未完成请求（状态 0x{status:02X}）")),
    }
}

fn query(device: &Device, command: (u8, u8, u8), args: &[u8]) -> Result<Vec<u8>, String> {
    let req = request(command.0, command.1, command.2, args)?;
    let mut last_error = String::new();
    for attempt in 0..5 {
        device.send(&req)?;
        thread::sleep(RESPONSE_WAIT);
        let response = device.receive(0, REPORT_LEN)?;
        match reply_body(&response, &req) {
            Ok(body) => return Ok(body.to_vec()),
            Err(error) => last_error = error,
        }
        if attempt < 4 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    Err(last_error)
}

/// A setting write is sent once; reissuing a timed-out write could duplicate a
/// state change. Callers must verify it with a separate read afterwards.
fn write(device: &Device, command: (u8, u8, u8), args: &[u8]) -> Result<(), String> {
    let req = request(command.0, command.1, command.2, args)?;
    device.send(&req)?;
    thread::sleep(RESPONSE_WAIT);
    let response = device.receive(0, REPORT_LEN)?;
    reply_body(&response, &req).map(|_| ())
}

fn read_battery(device: &Device) -> Result<u8, String> {
    query(device, BATTERY, &[])?
        .get(1)
        .copied()
        .ok_or_else(|| "Razer 电量回复字段缺失".into())
}

fn read_charging(device: &Device) -> Result<bool, String> {
    match query(device, CHARGING, &[])?.get(1).copied() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(value) => Err(format!("Razer 返回未知充电状态 0x{value:02X}")),
        None => Err("Razer 充电状态回复字段缺失".into()),
    }
}

fn read_dpi_raw(device: &Device) -> Result<Vec<u8>, String> {
    query(device, DPI_READ, &[0])
}

fn dpi_xy(raw: &[u8]) -> Result<(u16, u16), String> {
    if raw.len() != DPI_READ.2 as usize {
        return Err("Razer DPI 回复字段长度无效".into());
    }
    Ok((
        u16::from_be_bytes([raw[1], raw[2]]),
        u16::from_be_bytes([raw[3], raw[4]]),
    ))
}

fn read_dpi(device: &Device) -> Result<u16, String> {
    dpi_xy(&read_dpi_raw(device)?).map(|(x, _)| x)
}

fn read_rate_code(device: &Device) -> Result<u8, String> {
    match query(device, RATE_READ, &[])?.first().copied() {
        Some(code @ (1 | 2 | 8)) => Ok(code),
        Some(value) => Err(format!("Razer 返回未知回报率字段 0x{value:02X}")),
        None => Err("Razer 回报率回复字段缺失".into()),
    }
}

fn rate_from_code(code: u8) -> u16 {
    1000 / u16::from(code)
}

fn read_rate(device: &Device) -> Result<u16, String> {
    read_rate_code(device).map(rate_from_code)
}

fn original_dpi_write_args(raw: &[u8], dpi: u16) -> Result<Vec<u8>, String> {
    if raw.len() != DPI_WRITE.2 as usize {
        return Err("Razer 原始 DPI 字段长度无效".into());
    }
    let mut args = raw.to_vec();
    args[0] = 0;
    args[1..3].copy_from_slice(&dpi.to_be_bytes());
    args[3..5].copy_from_slice(&dpi.to_be_bytes());
    Ok(args)
}

fn read_stage_table(device: &Device) -> Result<Vec<u8>, String> {
    query(device, DPI_STAGES_READ, &[1])
}

/// Decode the 38-byte stage payload while retaining only valid, known DPI data.
fn parse_stages(body: &[u8]) -> Result<(Vec<u16>, u8), String> {
    if body.len() != 38 || !(1..=5).contains(&body[2]) || !(1..=body[2]).contains(&body[1]) {
        return Err("Razer DPI 档位结构未知".into());
    }
    let mut stages = Vec::with_capacity(body[2] as usize);
    for i in 0..body[2] as usize {
        let at = 3 + i * 7;
        let dpi = u16::from_be_bytes([body[at + 1], body[at + 2]]);
        if !(100..=30000).contains(&dpi) {
            return Err("Razer DPI 档位值异常".into());
        }
        stages.push(dpi);
    }
    Ok((stages, body[1] - 1))
}

/// Change the active stage without replacing any unused slots or reserved bytes.
fn update_active_stage(body: &[u8], dpi: u16) -> Result<Vec<u8>, String> {
    let (stages, active) = parse_stages(body)?;
    if !(100..=30000).contains(&dpi) || !dpi.is_multiple_of(50) {
        return Err("DPI 值超出该型号支持范围".into());
    }
    if active as usize >= stages.len() {
        return Err("Razer DPI 活动档位无效".into());
    }
    let mut out = body.to_vec();
    out[0] = 1;
    out[1] = active + 1;
    let at = 3 + active as usize * 7;
    let bytes = dpi.to_be_bytes();
    out[at + 1..at + 5].copy_from_slice(&[bytes[0], bytes[1], bytes[0], bytes[1]]);
    Ok(out)
}

fn same_stage_table(actual: &[u8], expected: &[u8]) -> bool {
    actual.len() == expected.len() && actual.get(1..) == expected.get(1..)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn base_state(present: bool) -> State {
    State {
        key: DEVICE_KEY.into(),
        present,
        wired: true,
        identity: "Razer Viper V2 Pro wired · USB 1532:00A5 · TID 0x1F".into(),
        settings_error: "DPI 与回报率可用；此有线型号没有已确认的改键或灯效设置。".into(),
        updated: now(),
        ..Default::default()
    }
}

fn read_from(device: &Device, mut state: State) -> State {
    match read_battery(device) {
        Ok(raw) => {
            state.percent = Some(((u16::from(raw) * 100 + 127) / 255) as u8);
            state.fresh = true;
        }
        Err(error) => state.error = error,
    }
    match read_charging(device) {
        Ok(value) => {
            state.charging = Some(value);
            state.fresh = true;
        }
        Err(error) if state.error.is_empty() => state.error = format!("充电状态不可用：{error}"),
        Err(_) => {}
    }
    match read_dpi(device) {
        Ok(value) => {
            state.dpi = Some(value);
            state.fresh = true;
        }
        Err(error) => append_setting_error(&mut state, format!("DPI：{error}")),
    }
    match read_rate(device) {
        Ok(value) => {
            state.rate = Some(value);
            state.fresh = true;
        }
        Err(error) => append_setting_error(&mut state, format!("回报率：{error}")),
    }
    state
}

fn append_setting_error(state: &mut State, message: String) {
    if state.settings_error.is_empty() {
        state.settings_error = message;
    } else {
        state.settings_error.push(' ');
        state.settings_error.push_str(&message);
    }
}

fn unique_device() -> Result<Option<Device>, String> {
    let interfaces = hid::enumerate()?;
    let matches: Vec<_> = interfaces
        .iter()
        .filter(|i| matching_interface(i))
        .collect();
    match matches.as_slice() {
        [] => Ok(None),
        [interface] => Device::open(&interface.path).map(Some),
        _ => Err("找到多个 Viper V2 Pro Feature 接口；为避免向错误接口发送请求，已停止操作".into()),
    }
}

/// Reads battery, charging, current DPI, and polling rate.
pub fn read_state() -> Result<State, String> {
    match unique_device()? {
        Some(device) => Ok(read_from(&device, base_state(true))),
        None => {
            let mut state = base_state(false);
            state.error =
                "未找到唯一的 Viper V2 Pro 有线 Feature 接口（1532:00A5 / MI_00 / 91 字节）".into();
            Ok(state)
        }
    }
}

fn rollback_dpi(
    device: &Device,
    original_dpi: &[u8],
    original_table: &[u8],
    restore_dpi: bool,
    restore_table: bool,
) -> Result<(), String> {
    let mut errors = Vec::new();
    if restore_table {
        let mut restore = original_table.to_vec();
        restore[0] = 1;
        if let Err(error) = write(device, DPI_STAGES_WRITE, &restore) {
            errors.push(format!("恢复原 DPI 档位表的写入未确认：{error}"));
        }
    }
    if restore_dpi {
        if let Err(error) = write(device, DPI_WRITE, original_dpi) {
            errors.push(format!("恢复原 DPI 的写入未确认：{error}"));
        }
    }
    thread::sleep(Duration::from_millis(180));
    if restore_table {
        match read_stage_table(device) {
            Ok(actual) if same_stage_table(&actual, original_table) => {}
            Ok(_) => errors.push("原 DPI 档位表读回不匹配".into()),
            Err(error) => errors.push(format!("原 DPI 档位表读回失败：{error}")),
        }
    }
    if restore_dpi {
        match read_dpi_raw(device) {
            Ok(actual) if actual == original_dpi => {}
            Ok(_) => errors.push("原 DPI 字段读回不匹配".into()),
            Err(error) => errors.push(format!("原 DPI 字段读回失败：{error}")),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("；"))
    }
}

fn rollback_rate(device: &Device, original_code: u8) -> Result<(), String> {
    write(device, RATE_WRITE, &[original_code])
        .map_err(|error| format!("恢复原回报率的写入未确认：{error}"))?;
    thread::sleep(Duration::from_millis(180));
    match read_rate_code(device) {
        Ok(actual) if actual == original_code => Ok(()),
        Ok(actual) => Err(format!(
            "原回报率读回不匹配：原字段 0x{original_code:02X}，实际 0x{actual:02X}"
        )),
        Err(error) => Err(format!("原回报率读回失败：{error}")),
    }
}

fn failed_with_rollback(label: &str, cause: String, rollback: Result<(), String>) -> String {
    match rollback {
        Ok(()) => format!("{label}设置未确认：{cause}；原设置已恢复并读回确认。"),
        Err(error) => {
            format!("{label}设置未确认：{cause}；恢复原设置也未确认：{error}；请重新读取设备。")
        }
    }
}

/// Applies only the two settings confirmed for this wired model and checks readback.
pub fn apply_setting(action: Action) -> Result<(), String> {
    let device = unique_device()?.ok_or("设备尚未连接")?;
    match action {
        Action::Dpi(dpi) if (100..=30000).contains(&dpi) && dpi.is_multiple_of(50) => {
            let original_dpi = read_dpi_raw(&device)
                .map_err(|error| format!("读取原 DPI 失败，未发送写入：{error}"))?;
            let (original_x, original_y) = dpi_xy(&original_dpi)?;
            if !(100..=30000).contains(&original_x) || !(100..=30000).contains(&original_y) {
                return Err("设备当前 DPI 字段超出已知范围，未发送写入".into());
            }
            let original_table = read_stage_table(&device)
                .map_err(|error| format!("读取原 DPI 档位表失败，未发送写入：{error}"))?;
            parse_stages(&original_table)
                .map_err(|error| format!("原 DPI 档位表无法安全保留，未发送写入：{error}"))?;
            let updated_table = update_active_stage(&original_table, dpi)?;
            if let Err(error) = write(&device, DPI_STAGES_WRITE, &updated_table) {
                let rollback = rollback_dpi(&device, &original_dpi, &original_table, true, true);
                return Err(failed_with_rollback("DPI ", error, rollback));
            }
            let target_dpi = original_dpi_write_args(&original_dpi, dpi)?;
            if let Err(error) = write(&device, DPI_WRITE, &target_dpi) {
                let rollback = rollback_dpi(&device, &original_dpi, &original_table, true, true);
                return Err(failed_with_rollback("DPI ", error, rollback));
            }
            thread::sleep(Duration::from_millis(180));
            let verified = (|| {
                let actual_dpi = read_dpi_raw(&device)?;
                let (x, y) = dpi_xy(&actual_dpi)?;
                if x != dpi || y != dpi {
                    return Err(format!("DPI 读回不匹配：期望 X/Y 为 {dpi}，实际为 {x}/{y}"));
                }
                let actual_table = read_stage_table(&device)?;
                if !same_stage_table(&actual_table, &updated_table) {
                    return Err("DPI 档位表读回不匹配".into());
                }
                Ok(())
            })();
            if let Err(error) = verified {
                let rollback = rollback_dpi(&device, &original_dpi, &original_table, true, true);
                return Err(failed_with_rollback("DPI ", error, rollback));
            }
            Ok(())
        }
        Action::Rate(rate) if [125, 500, 1000].contains(&rate) => {
            let original_code = read_rate_code(&device)
                .map_err(|error| format!("读取原回报率失败，未发送写入：{error}"))?;
            let code = match rate {
                1000 => 1,
                500 => 2,
                125 => 8,
                _ => unreachable!(),
            };
            if let Err(error) = write(&device, RATE_WRITE, &[code]) {
                return Err(failed_with_rollback(
                    "回报率 ",
                    error,
                    rollback_rate(&device, original_code),
                ));
            }
            thread::sleep(Duration::from_millis(180));
            let verified = read_rate_code(&device).and_then(|actual| {
                if actual == code {
                    Ok(())
                } else {
                    Err(format!(
                        "回报率读回不匹配：期望 {rate} Hz，实际 {} Hz",
                        rate_from_code(actual)
                    ))
                }
            });
            if let Err(error) = verified {
                return Err(failed_with_rollback(
                    "回报率 ",
                    error,
                    rollback_rate(&device, original_code),
                ));
            }
            Ok(())
        }
        Action::Dpi(_) => Err("DPI 必须为 100–30000，且是 50 的倍数".into()),
        Action::Rate(_) => Err("回报率仅支持 125、500 或 1000 Hz".into()),
        _ => Err("此型号目前只开放 DPI 和回报率设置".into()),
    }
}

/// Parses the host's externally-tagged `Action` JSON and applies supported settings.
pub fn apply_setting_value(value: &serde_json::Value) -> Result<(), String> {
    let action = serde_json::from_value::<Action>(value.clone())
        .map_err(|e| format!("驱动参数 action 无效：{e}"))?;
    apply_setting(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn success_reply(request: &[u8; REPORT_LEN], body: &[u8]) -> [u8; REPORT_LEN] {
        let mut wire = *request;
        wire[1] = 2;
        wire[8] |= 0x80;
        wire[9..9 + body.len()].copy_from_slice(body);
        wire[89] = checksum(&wire);
        wire
    }

    #[test]
    fn migrated_dpi_and_rate_requests_match_the_core_schema() {
        let dpi = request(
            DPI_WRITE.0,
            DPI_WRITE.1,
            DPI_WRITE.2,
            &[0, 0x04, 0xb0, 0x04, 0xb0, 0, 0],
        )
        .unwrap();
        assert_eq!(
            &dpi[..16],
            &[0, 0, 0x1f, 0, 0, 0, 7, 4, 5, 0, 4, 0xb0, 4, 0xb0, 0, 0]
        );
        let dpi_read = request(DPI_READ.0, DPI_READ.1, DPI_READ.2, &[0]).unwrap();
        let stages_read = request(
            DPI_STAGES_READ.0,
            DPI_STAGES_READ.1,
            DPI_STAGES_READ.2,
            &[1],
        )
        .unwrap();
        let stages_write = request(
            DPI_STAGES_WRITE.0,
            DPI_STAGES_WRITE.1,
            DPI_STAGES_WRITE.2,
            &[1; 38],
        )
        .unwrap();
        let rate = request(RATE_WRITE.0, RATE_WRITE.1, RATE_WRITE.2, &[2]).unwrap();
        assert_eq!(&rate[..10], &[0, 0, 0x1f, 0, 0, 0, 1, 0, 5, 2]);
        assert_eq!((dpi_read[9], stages_read[9], stages_write[9]), (0, 1, 1));
        assert_eq!(dpi[89], checksum(&dpi));
        assert_eq!(rate[89], checksum(&rate));
    }

    #[test]
    fn readback_decodes_only_the_documented_dpi_and_polling_codes() {
        let dpi_req = request(DPI_READ.0, DPI_READ.1, DPI_READ.2, &[0]).unwrap();
        let dpi_reply = success_reply(&dpi_req, &[1, 0x04, 0xb0, 0x04, 0xb0, 0, 0]);
        let dpi_body = reply_body(&dpi_reply, &dpi_req).unwrap();
        assert_eq!(u16::from_be_bytes([dpi_body[1], dpi_body[2]]), 1200);

        let rate_req = request(RATE_READ.0, RATE_READ.1, RATE_READ.2, &[]).unwrap();
        let rate_reply = success_reply(&rate_req, &[2]);
        assert_eq!(reply_body(&rate_reply, &rate_req).unwrap(), &[2]);
        assert_eq!([1u8, 2, 8].map(|code| 1000 / code as u16), [1000, 500, 125]);
    }

    #[test]
    fn active_stage_update_preserves_other_stages_and_reserved_bytes() {
        let mut table = vec![0; 38];
        table[1] = 2;
        table[2] = 2;
        table[3..10].copy_from_slice(&[1, 1, 144, 1, 144, 7, 9]);
        table[10..17].copy_from_slice(&[2, 3, 32, 3, 32, 6, 8]);
        table[30..].copy_from_slice(&[9, 8, 7, 6, 5, 4, 3, 2]);
        let edited = update_active_stage(&table, 1200).unwrap();
        assert_eq!(&edited[11..15], &[4, 176, 4, 176]);
        assert_eq!(&edited[3..10], &table[3..10]);
        assert_eq!(&edited[10], &table[10]);
        assert_eq!(&edited[15..], &table[15..]);
        assert_eq!(edited[0], 1);
        assert!(update_active_stage(&table, 1201).is_err());
    }

    #[test]
    fn dpi_write_changes_only_xy_and_storage_selector() {
        let original = [0, 0x03, 0x20, 0x04, 0xb0, 0x7a, 0x6b];
        let edited = original_dpi_write_args(&original, 1200).unwrap();
        assert_eq!(edited, [0, 0x04, 0xb0, 0x04, 0xb0, 0x7a, 0x6b]);
    }

    #[test]
    fn response_must_match_transaction_checksum_and_command() {
        let request = request(BATTERY.0, BATTERY.1, BATTERY.2, &[]).unwrap();
        let reply = success_reply(&request, &[0, 200]);
        assert_eq!(reply_body(&reply, &request).unwrap(), &[0, 200]);
        let mut stale = reply;
        stale[2] ^= 1;
        stale[89] = checksum(&stale);
        assert!(reply_body(&stale, &request).is_err());
    }
}
