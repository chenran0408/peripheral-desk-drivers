//! Exact-model G305 HID++ 2.0 adapter. DPI writes are guarded and reversible.
//! Protocol reference: Solaar e7304c4c451cc9bb4f206a914844525e67856a28,
//! lib/logitech_receiver/{base,hidpp20}.py (GPL-2.0-or-later).
//! Rust implementation and Win32 transport are authored for this AGPLv3 project.
use std::time::{SystemTime, UNIX_EPOCH};
pub type Result<T> = std::result::Result<T, String>;
pub use peripheral_desk::model::Action;
mod transport;

pub const G305_KEY: &str = "logitech-g305";
pub const G305_NAME: &str = "G305 Lightspeed Wireless Gaming Mouse";
const SLOT: u8 = 1;
const SOFTWARE_ID: u8 = 0x0c;

/// Transport must serialize exchanges and ignore unrelated input reports.
pub trait Transport {
    fn exchange(&mut self, request: &[u8; 20]) -> Result<Vec<u8>>;
}

fn request(feature: u8, function: u8, args: &[u8]) -> [u8; 20] {
    assert!(function & 0x0f == 0 && args.len() <= 16);
    let mut out = [0; 20];
    out[..4].copy_from_slice(&[0x11, SLOT, feature, function | SOFTWARE_ID]);
    out[4..4 + args.len()].copy_from_slice(args);
    out
}

/// None denotes a different client's reply or an unsolicited notification.
pub fn matching_reply(wire: &[u8], req: &[u8; 20]) -> Result<Option<Vec<u8>>> {
    let size = match wire.first() {
        Some(0x10) => 7,
        Some(0x11) => 20,
        _ => return Ok(None),
    };
    if wire.len() < size || wire[1] != req[1] {
        return Ok(None);
    }
    if wire[2] == 0xff || wire[2] == 0x8f {
        if wire[3..5] == req[2..4] {
            return Err(format!(
                "HID++ 查询被拒绝，错误码 0x{:02X}（设备可能休眠）",
                wire[5]
            ));
        }
        return Ok(None);
    }
    if wire[2..4] != req[2..4] {
        return Ok(None);
    }
    Ok(Some(wire[4..size].to_vec()))
}

struct Reader<T>(T);
impl<T: Transport> Reader<T> {
    fn call(&mut self, feature: u8, function: u8, args: &[u8]) -> Result<Vec<u8>> {
        self.0.exchange(&request(feature, function, args))
    }
    fn feature(&mut self, id: u16) -> Result<u8> {
        self.feature_info(id).map(|(index, _)| index)
    }
    fn feature_info(&mut self, id: u16) -> Result<(u8, u8)> {
        let reply = self.call(0, 0, &id.to_be_bytes())?;
        if reply.len() < 3 {
            return Err("HID++ 特性回复不完整".into());
        }
        if reply[1] & 0xc0 != 0 {
            return Err(format!("拒绝访问隐藏或已弃用特性 0x{id:04X}"));
        }
        let index = reply[0];
        if index == 0 {
            return Err(format!("设备没有公开特性 0x{id:04X}"));
        }
        Ok((index, reply[2]))
    }
    fn name(&mut self, index: u8) -> Result<String> {
        let count = *self.call(index, 0, &[])?.first().ok_or("设备名长度为空")? as usize;
        if count == 0 || count > 64 {
            return Err("设备名长度超出安全范围".into());
        }
        let mut bytes = Vec::with_capacity(count);
        while bytes.len() < count {
            let part = self.call(index, 0x10, &[bytes.len() as u8])?;
            let size = part.len().min(count - bytes.len());
            if size == 0 {
                return Err("设备名片段为空".into());
            }
            bytes.extend_from_slice(&part[..size]);
        }
        String::from_utf8(bytes).map_err(|_| "设备名不是 UTF-8".into())
    }
}

const DPI_FEATURE: u16 = 0x2201;
const ONBOARD_FEATURE: u16 = 0x8100;

fn dpi_list<T: Transport>(reader: &mut Reader<T>, feature: u8) -> Result<Vec<u16>> {
    let mut bytes = Vec::new();
    for chunk in 0..16u8 {
        let reply = reader.call(feature, 0x10, &[0, 0, chunk])?;
        if reply.len() < 3 {
            return Err("DPI 档位回复不完整".into());
        }
        bytes.extend_from_slice(&reply[1..]);
        if bytes.len() >= 2 && bytes[bytes.len() - 2..] == [0, 0] {
            break;
        }
        if chunk == 15 {
            return Err("DPI 档位列表超过安全读取上限".into());
        }
    }
    // Long HID++ replies have an odd number of bytes after the sensor ID.
    // Zero termination ends the entries before the remaining padding.
    let mut values: Vec<u16> = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let value = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
        if value == 0 {
            break;
        }
        if value >> 13 == 0b111 {
            let step = value & 0x1fff;
            if step == 0 || values.is_empty() || i + 3 >= bytes.len() {
                return Err("设备返回了无法安全解释的 DPI 范围".into());
            }
            let end = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]);
            let start = values[values.len() - 1].saturating_add(step);
            if end < start
                || (end - start) % step != 0
                || values.len() + ((end - start) / step) as usize > 512
            {
                return Err("设备返回了无效或过大的 DPI 范围".into());
            }
            let mut next = start;
            while next <= end {
                values.push(next);
                let Some(following) = next.checked_add(step) else {
                    break;
                };
                next = following;
            }
            i += 4;
        } else {
            values.push(value);
            i += 2;
        }
        if values.len() > 512 {
            return Err("DPI 档位数量超过安全上限".into());
        }
    }
    if values.is_empty() || values.windows(2).any(|w| w[0] >= w[1]) {
        return Err("设备没有提供有效的 DPI 档位".into());
    }
    Ok(values)
}

fn read_dpi<T: Transport>(reader: &mut Reader<T>, feature: u8, values: &[u16]) -> Result<u16> {
    let body = reader.call(feature, 0x20, &[0])?;
    if body.len() < 5 {
        return Err("当前 DPI 回复不完整".into());
    }
    let current = u16::from_be_bytes([body[1], body[2]]);
    let value = if current == 0 {
        u16::from_be_bytes([body[3], body[4]])
    } else {
        current
    };
    if !values.contains(&value) {
        return Err(format!("设备当前 DPI {value} 不在其公开档位列表中"));
    }
    Ok(value)
}

fn onboard_mode<T: Transport>(reader: &mut Reader<T>) -> Result<u8> {
    let (feature, version) = reader.feature_info(ONBOARD_FEATURE)?;
    if version != 0 {
        return Err(format!("不支持的板载配置特性版本 {version}"));
    }
    let body = reader.call(feature, 0x20, &[])?;
    body.first()
        .copied()
        .ok_or_else(|| "板载配置模式回复为空".into())
}

fn verify_session<T: Transport>(transport: T) -> Result<(Reader<T>, u8, Vec<u8>)> {
    let mut reader = Reader(transport);
    let mark = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u8;
    let ping = reader.call(0, 0x10, &[0, 0, mark])?;
    if ping.len() < 3 || ping[0..2] != [4, 2] || ping[2] != mark {
        return Err("G305 需要 HID++ 4.2 且 ping 必须回显本次标记".into());
    }
    let identity_feature = reader.feature(0x0003)?;
    let ids = reader.call(identity_feature, 0, &[])?;
    let name_feature = reader.feature(0x0005)?;
    let name = reader.name(name_feature)?;
    verify_model(&ids, &name)?;
    Ok((reader, identity_feature, ids))
}

/// Public 0x0003 transport IDs are authoritative; receiver VID/PID alone is not.
fn verify_model(ids: &[u8], name: &str) -> Result<()> {
    if ids.len() < 13 {
        return Err("设备身份回复不完整".into());
    }
    let flags = ids[6];
    if flags & 0xf0 != 0 {
        return Err("设备身份包含未知 transport 标志".into());
    }
    let mut offset = 7;
    let mut wpid = None;
    for bit in [1, 2, 4, 8] {
        if flags & bit != 0 {
            let pair = ids
                .get(offset..offset + 2)
                .ok_or("设备 transport ID 越界")?;
            if bit == 4 {
                wpid = Some(u16::from_be_bytes([pair[0], pair[1]]));
            }
            offset += 2;
        }
    }
    if wpid != Some(0x4074) || name != G305_NAME {
        return Err(format!(
            "接收器配对型号不匹配：{name} / WPID {}；本驱动仅适配 G305 4074",
            wpid.map(|v| format!("{v:04X}"))
                .unwrap_or_else(|| "unknown".into())
        ));
    }
    Ok(())
}
fn battery(body: &[u8]) -> Result<(Option<u8>, Option<bool>)> {
    if body.len() < 3 {
        return Err("电量回复不完整".into());
    }
    if body[0] > 100 || body[1] > 100 {
        return Err("电量字段超出 0–100，拒绝猜测".into());
    }
    let charging = match body[2] {
        0 => Some(false),
        1..=4 => Some(true),
        5 => return Err("设备报告电池异常".into()),
        6 => return Err("设备报告电池温度异常".into()),
        _ => return Err(format!("未知电池状态 0x{:02X}", body[2])),
    };
    // HID++ 0x1000 uses zero for an unavailable percentage, not a measured 0%.
    Ok(((body[0] != 0).then_some(body[0]), charging))
}

pub fn read_with<T: Transport>(transport: T) -> Result<peripheral_desk::model::State> {
    let (mut reader, identity_feature, ids) = verify_session(transport)?;
    let reading = reader
        .feature(0x1000)
        .and_then(|index| reader.call(index, 0, &[]))
        .and_then(|body| battery(&body));
    let (percent, charging, error, fresh) = match reading {
        Ok((percent, charging)) => (percent, charging, String::new(), true),
        Err(error) => (None, None, error, false),
    };
    let mut firmware = Vec::new();
    // Only the two public application/bootloader entries, never hidden features.
    for layer in 0..ids[0].min(2) {
        if let Ok(body) = reader.call(identity_feature, 0x10, &[layer]) {
            if body.len() >= 8
                && body[0] & 0x0f <= 1
                && body[1..4].iter().all(u8::is_ascii_alphanumeric)
            {
                firmware.push(format!(
                    "{} {:02X}.{:02X}.B{:04X}",
                    String::from_utf8_lossy(&body[1..4]),
                    body[4],
                    body[5],
                    u16::from_be_bytes([body[6], body[7]])
                ));
            }
        }
    }
    let (dpi, dpi_stages, mode_note) = match reader.feature_info(DPI_FEATURE)
        .and_then(|(index, version)| {
            if version != 1 { return Err(format!("G305 的 DPI 特性版本不是已验证的 1（读到 {version}）")); }
            let stages = dpi_list(&mut reader, index)?;
            let current = read_dpi(&mut reader, index, &stages)?;
            let (current, stages, note) = match onboard_mode(&mut reader)? {
                2 => (Some(current), stages, "板载配置已关闭；支持写入当前 DPI。".to_string()),
                1 => (None, Vec::new(), "DPI 设置不可用：板载配置正在控制 DPI。为避免改写板载配置或写入无效，请先在罗技软件中关闭板载配置。".to_string()),
                v => return Err(format!("未知板载配置模式 0x{v:02X}，拒绝开放 DPI 写入")),
            };
            Ok((current, stages, note))
        }) {
            Ok((current, stages, note)) => (current, stages, note),
            Err(e) => (None, Vec::new(), format!("DPI 读取失败：{e}")),
        };
    let active_stage = dpi
        .and_then(|value| dpi_stages.iter().position(|v| *v == value))
        .map(|v| v as u8);
    Ok(peripheral_desk::model::State {
        key: G305_KEY.into(), present: true, fresh, percent, charging, error, wired: false,
        updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        identity: format!("046D:C53F / WPID 4074 / HID++ 4.2 / {}", if firmware.is_empty() {"firmware unknown".into()} else {firmware.join("; ")}),
        dpi, dpi_stages, active_stage,
        settings_error: format!("公开实现适配未实测；电量可能按固件档位跳变。{mode_note}仅 DPI 可写；回报率、按键、宏、灯光和固件命令未开放。"),
        ..Default::default()
    })
}

fn set_dpi<T: Transport>(reader: &mut Reader<T>, feature: u8, dpi: u16) -> Result<()> {
    reader
        .call(feature, 0x30, &[0, (dpi >> 8) as u8, dpi as u8])
        .map(|_| ())
}

pub fn apply_action_with<T: Transport>(
    transport: T,
    action: peripheral_desk::model::Action,
) -> Result<()> {
    let (mut reader, _, _) = verify_session(transport)?;
    let mode = onboard_mode(&mut reader)?;
    if mode != 2 {
        return Err(format!(
            "板载配置模式为 {mode}；未修改设备。请先在罗技软件中关闭板载配置，再重试。"
        ));
    }
    let (feature, version) = reader.feature_info(DPI_FEATURE)?;
    if version != 1 {
        return Err(format!("不支持的 DPI 特性版本 {version}"));
    }
    let choices = dpi_list(&mut reader, feature)?;
    let dpi = match action {
        peripheral_desk::model::Action::Dpi(value) if choices.contains(&value) => value,
        peripheral_desk::model::Action::Stage(index) => *choices
            .get(index as usize)
            .ok_or_else(|| "DPI 档位编号超出设备公开列表".to_string())?,
        peripheral_desk::model::Action::Dpi(value) => {
            return Err(format!("DPI {value} 不在设备公开支持的档位中"))
        }
        _ => return Err("G305 仅支持精确设备 DPI 设置；其他动作未开放".into()),
    };
    let original = read_dpi(&mut reader, feature, &choices)?;
    if original == dpi {
        return Ok(());
    }
    let result = set_dpi(&mut reader, feature, dpi).and_then(|_| {
        let actual = read_dpi(&mut reader, feature, &choices)?;
        if actual == dpi {
            Ok(())
        } else {
            Err(format!("DPI 写入后读回 {actual}，预期 {dpi}"))
        }
    });
    if let Err(error) = result {
        let rollback = set_dpi(&mut reader, feature, original)
            .and_then(|_| read_dpi(&mut reader, feature, &choices))
            .and_then(|actual| {
                if actual == original {
                    Ok(())
                } else {
                    Err(format!("恢复读回为 {actual}，预期 {original}"))
                }
            });
        return match rollback {
            Ok(()) => Err(format!("DPI 设置失败，已恢复原值 {original}：{error}")),
            Err(restore) => Err(format!(
                "DPI 设置失败：{error}；恢复原值 {original} 失败：{restore}"
            )),
        };
    }
    Ok(())
}

pub fn apply_dpi_with<T: Transport>(transport: T, dpi: u16) -> Result<()> {
    apply_action_with(transport, peripheral_desk::model::Action::Dpi(dpi))
}

pub fn apply_g305_setting(action: peripheral_desk::model::Action) -> Result<()> {
    if !matches!(
        action,
        peripheral_desk::model::Action::Dpi(_) | peripheral_desk::model::Action::Stage(_)
    ) {
        return Err("G305 仅支持精确设备 DPI 设置；其他动作未开放".into());
    }
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut last = "未找到 G305 的 046D:C53F HID++ 长报文接口".to_string();
    for i in interfaces.iter().filter(|i| {
        i.vid == 0x046d
            && i.pid == 0xc53f
            && i.usage_page == 0xff00
            && i.usage == 2
            && i.input_len == 20
            && i.output_len == 20
    }) {
        match transport::WinTransport::open(&i.path).and_then(|t| apply_action_with(t, action)) {
            Ok(()) => return Ok(()),
            Err(error) => last = error,
        }
    }
    Err(last)
}

pub fn read_g305() -> Result<peripheral_desk::model::State> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut last =
        "未找到 G305 的 046D:C53F HID++ 长报文接口；仅支持该接收器，不能用共享 VID/PID 推断型号"
            .to_string();
    for i in interfaces.iter().filter(|i| {
        i.vid == 0x046d
            && i.pid == 0xc53f
            && i.usage_page == 0xff00
            && i.usage == 2
            && i.input_len == 20
            && i.output_len == 20
    }) {
        match transport::WinTransport::open(&i.path).and_then(read_with) {
            Ok(state) => return Ok(state),
            Err(error) => last = error,
        }
    }
    Ok(peripheral_desk::model::State {
        key: G305_KEY.into(),
        present: false,
        fresh: false,
        error: last,
        settings_error: "公开实现适配未实测；接收器存在不代表其配对型号已确认。".into(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_is_exact_not_receiver_wide() {
        let mut ids = [0; 16];
        ids[6] = 4;
        ids[7..9].copy_from_slice(&[0x40, 0x74]);
        assert!(verify_model(&ids, G305_NAME).is_ok());
        assert!(verify_model(&ids, "G304 Lightspeed Wireless Gaming Mouse").is_err());
        ids[8] = 0x93;
        assert!(verify_model(&ids, G305_NAME).is_err());
        assert!(verify_model(&[0; 8], G305_NAME).is_err());
    }
    #[test]
    fn filters_notifications_clients_and_errors() {
        let req = request(5, 0, &[]);
        let mut reply = req;
        reply[4..7].copy_from_slice(&[50, 30, 0]);
        assert_eq!(
            matching_reply(&reply, &req).unwrap().unwrap()[..3],
            [50, 30, 0]
        );
        reply[3] = 0;
        assert!(matching_reply(&reply, &req).unwrap().is_none());
        reply[3] = 0x0d;
        assert!(matching_reply(&reply, &req).unwrap().is_none());
        reply[..6].copy_from_slice(&[0x10, 1, 0xff, 5, SOFTWARE_ID, 9]);
        assert!(matching_reply(&reply, &req).is_err());
        assert!(matching_reply(&reply[..5], &req).unwrap().is_none());
    }
    struct Fixture {
        wrong_name: bool,
        battery_failed: bool,
        seen_battery: bool,
        dpi: u16,
        mode: u8,
        writes: usize,
    }
    impl Transport for &mut Fixture {
        fn exchange(&mut self, req: &[u8; 20]) -> Result<Vec<u8>> {
            assert_eq!(req[0..2], [0x11, 1]);
            assert_eq!(req[3] & 15, SOFTWARE_ID);
            let mut out = vec![0; 16];
            match (req[2], req[3] & 0xf0) {
                (0, 0x10) => out[..3].copy_from_slice(&[4, 2, req[6]]),
                (0, 0) => {
                    out[0] = match u16::from_be_bytes([req[4], req[5]]) {
                        0x0003 => 2,
                        0x0005 => 3,
                        0x1000 => 5,
                        0x2201 => 6,
                        0x8100 => 7,
                        _ => panic!("unexpected feature"),
                    };
                    if u16::from_be_bytes([req[4], req[5]]) == DPI_FEATURE {
                        out[2] = 1;
                    }
                }
                (2, 0) => {
                    out[0] = 2;
                    out[6] = 4;
                    out[7..9].copy_from_slice(&[0x40, 0x74]);
                }
                (2, 0x10) => {
                    out[..8].copy_from_slice(&[0, b'R', b'Q', b'M', 0x68, 2, 0, 0x21]);
                }
                (3, 0) => out[0] = G305_NAME.len() as u8,
                (3, 0x10) => {
                    let name = if self.wrong_name {
                        "G304 Lightspeed Wireless Gaming Mouse"
                    } else {
                        G305_NAME
                    };
                    let part = &name.as_bytes()[req[4] as usize..];
                    let count = part.len().min(16);
                    out[..count].copy_from_slice(&part[..count]);
                }
                (5, 0) => {
                    self.seen_battery = true;
                    if self.battery_failed {
                        return Err("fixture battery timeout".into());
                    }
                    out[..3].copy_from_slice(&[50, 30, 0]);
                }
                (6, 0x10) => {
                    assert_eq!(&req[4..7], &[0, 0, 0]);
                    out[..7].copy_from_slice(&[0, 0x03, 0x20, 0x06, 0x40, 0, 0]);
                }
                (6, 0x20) => out[..5].copy_from_slice(&[
                    0,
                    (self.dpi >> 8) as u8,
                    self.dpi as u8,
                    0x06,
                    0x40,
                ]),
                (6, 0x30) => {
                    self.writes += 1;
                    self.dpi = u16::from_be_bytes([req[5], req[6]]);
                }
                (7, 0x20) => out[0] = self.mode,
                _ => panic!("unexpected G305 fixture operation"),
            }
            Ok(out)
        }
    }
    #[test]
    fn complete_read_gates_battery_on_model_and_preserves_identity_on_failure() {
        let mut good = Fixture {
            wrong_name: false,
            battery_failed: false,
            seen_battery: false,
            dpi: 1600,
            mode: 2,
            writes: 0,
        };
        let state = read_with(&mut good).unwrap();
        assert!(state.present && state.fresh);
        assert_eq!(state.percent, Some(50));
        assert_eq!(state.dpi, Some(1600));
        assert_eq!(state.dpi_stages, vec![800, 1600]);
        assert!(state.identity.contains("WPID 4074"));
        let mut bad = Fixture {
            wrong_name: true,
            battery_failed: false,
            seen_battery: false,
            dpi: 1600,
            mode: 2,
            writes: 0,
        };
        assert!(read_with(&mut bad).is_err());
        assert!(!bad.seen_battery);
        let mut sleeping = Fixture {
            wrong_name: false,
            battery_failed: true,
            seen_battery: false,
            dpi: 1600,
            mode: 2,
            writes: 0,
        };
        let state = read_with(&mut sleeping).unwrap();
        assert!(state.present && !state.fresh);
        assert_eq!(state.percent, None);
        assert!(state.error.contains("timeout"));
    }
    #[test]
    fn unknown_is_not_fabricated_battery() {
        assert_eq!(battery(&[90, 50, 0]).unwrap(), (Some(90), Some(false)));
        assert_eq!(battery(&[0, 0, 0]).unwrap(), (None, Some(false)));
        assert!(battery(&[101, 0, 0]).is_err());
        assert!(battery(&[50, 0, 9]).is_err());
    }

    #[test]
    fn dpi_write_requires_disabled_profiles_and_valid_stage() {
        let mut fixture = Fixture {
            wrong_name: false,
            battery_failed: false,
            seen_battery: false,
            dpi: 1600,
            mode: 1,
            writes: 0,
        };
        assert!(apply_dpi_with(&mut fixture, 1600)
            .unwrap_err()
            .contains("未修改设备"));
        assert_eq!(fixture.writes, 0);
        fixture.mode = 2;
        assert!(apply_dpi_with(&mut fixture, 400).is_err());
        assert_eq!(fixture.writes, 0);
        apply_dpi_with(&mut fixture, 800).unwrap();
        assert_eq!(fixture.dpi, 800);
        assert_eq!(fixture.writes, 1);
    }

    #[test]
    fn active_profile_hides_unavailable_dpi_controls() {
        let mut fixture = Fixture {
            wrong_name: false,
            battery_failed: false,
            seen_battery: false,
            dpi: 1600,
            mode: 1,
            writes: 0,
        };
        let state = read_with(&mut fixture).unwrap();
        assert_eq!(state.dpi, None);
        assert!(state.dpi_stages.is_empty());
        assert_eq!(state.active_stage, None);
        assert!(state.settings_error.contains("板载配置正在控制 DPI"));
    }
}
