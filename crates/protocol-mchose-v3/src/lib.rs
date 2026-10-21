//! A7 V3 Ultra+ receiver adapter.
//!
//! Protocol facts: OpenMouse mouse-protocol bac031567b28f2a55c616fc525b03f2d5fa452cc,
//! src/mchose/v3.ts, src/drivers/mchose/v3-hid.ts and docs/mchose-protocol.md.
//! This Rust codec is independently authored; no upstream implementation code is copied.
use peripheral_desk::{
    controls::{Binding, BindingAction, ControlAction, ControlState},
    model::{Action, State},
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const VID: u16 = 0x3837;
const RECEIVER_PID: u16 = 0x1018;
const USAGE_PAGE: u16 = 0xff01;
const USAGE: u16 = 0x0001;
const REPORT_ID: u8 = 0x4d;
const REPORT_LEN: usize = 64;
const BODY_LEN: usize = REPORT_LEN - 1;
const INFO_COMMAND: u16 = 0x0900;
const SETTINGS_COMMAND: u16 = 0x0002;
const WRITE_SETTINGS_COMMAND: u16 = 0x0102;
const DPI_COMMAND: u16 = 0x0003;
const WRITE_DPI_COMMAND: u16 = 0x0103;
const BUTTONS_COMMAND: u16 = 0x0001;
const WRITE_BUTTONS_COMMAND: u16 = 0x0101;
const PRODUCT_NAME: &str = "MCHOSE A7 V3 Ultra+";
const MOUSE_PRODUCT_ID: u16 = 0x4026;
const MAX_DPI: u16 = 50_000;
const DPI_STEP: u16 = 50;
const WRITE_SETTLE: Duration = Duration::from_millis(400);
pub const DEVICE_KEY: &str = "mchose-a7-v3-ultra-plus";

type Result<T> = std::result::Result<T, String>;

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |sum, byte| sum ^ byte)
}

fn request() -> [u8; REPORT_LEN] {
    build_command(INFO_COMMAND, &[])
}

fn build_command(command: u16, data: &[u8]) -> [u8; REPORT_LEN] {
    let mut wire = [0; REPORT_LEN];
    wire[0] = REPORT_ID;
    let body = &mut wire[1..];
    body[0] = 1;
    body[1] = 1;
    body[2] = data.len() as u8;
    body[3..5].copy_from_slice(&command.to_le_bytes());
    body[5] = 0;
    body[6] = 0;
    body[7..7 + data.len()].copy_from_slice(data);
    body[7 + data.len()] = checksum(&body[1..7 + data.len()]);
    wire
}

fn reply_info(wire: &[u8], request: &[u8; REPORT_LEN]) -> Result<[u8; 16]> {
    if wire.len() != BODY_LEN + 1 || wire[0] != REPORT_ID {
        return Err("MCHOSE 输入报告长度或 Report ID 不匹配".into());
    }
    let body = &wire[1..];
    if body[0] != 1 || body[1] != 1 {
        return Err("MCHOSE 回复协议版本或校验标志不匹配".into());
    }
    let len = body[2] as usize;
    if body[3..5] != INFO_COMMAND.to_le_bytes() {
        return Err("MCHOSE 回复命令不匹配".into());
    }
    if body[5] != request[6] || body[6] != request[7] {
        return Err("MCHOSE 回复事务标识不匹配".into());
    }
    if len > BODY_LEN - 8 || body[7 + len] != checksum(&body[1..7 + len]) {
        return Err("MCHOSE 回复校验不匹配".into());
    }
    if len == 1 && body[7] == 0xff {
        return Err("MCHOSE 接收器已响应，但鼠标当前不可达".into());
    }
    if len != 16 {
        return Err("MCHOSE 身份数据长度不匹配".into());
    }
    body[7..7 + len]
        .try_into()
        .map_err(|_| "MCHOSE 身份数据不完整".into())
}

fn verify_identity(data: &[u8; 16], product: &str) -> Result<(u8, bool, bool)> {
    let vendor = u16::from_le_bytes([data[0], data[1]]);
    let product_id = u16::from_le_bytes([data[2], data[3]]);
    if vendor != VID || product_id != MOUSE_PRODUCT_ID || product != PRODUCT_NAME {
        return Err(format!(
            "MCHOSE 接收器后的型号身份不匹配：内部 {:04X}:{:04X}，产品名 {:?}",
            vendor, product_id, product
        ));
    }
    let charge = match data[11] {
        0 => false,
        1 => true,
        value => return Err(format!("未知 MCHOSE 充电状态 0x{value:02X}")),
    };
    if data[12] > 100 {
        return Err(format!("MCHOSE 电量超出 0–100：{}", data[12]));
    }
    let connected = match data[10] {
        0 => false,
        2 => true,
        value => return Err(format!("未知 MCHOSE 接收器连接状态 0x{value:02X}")),
    };
    Ok((data[12], charge, connected))
}

fn transact(transport: &mut WinTransport, command: u16, data: &[u8]) -> Result<Vec<u8>> {
    let request = build_command(command, data);
    transport.flush()?;
    transport.write(&request, Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(1500);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!("MCHOSE 命令 0x{command:04X} 超时"));
        }
        let wire = transport.read(REPORT_LEN, remaining)?;
        if wire.is_empty() {
            continue;
        }
        if wire.len() != REPORT_LEN || wire[0] != REPORT_ID {
            continue;
        }
        let body = &wire[1..];
        if body[0] != 1
            || body[1] != 1
            || body[3..5] != command.to_le_bytes()
            || body[5] != request[6]
            || body[6] != request[7]
        {
            continue;
        }
        let len = body[2] as usize;
        if len > BODY_LEN - 8 || body[7 + len] != checksum(&body[1..7 + len]) {
            return Err("MCHOSE 回复长度或校验不匹配".into());
        }
        if len == 1 && body[7] == 0xff {
            return Err("MCHOSE 接收器后的鼠标当前不可达".into());
        }
        return Ok(body[7..7 + len].to_vec());
    }
}

fn settings(transport: &mut WinTransport) -> Result<Vec<u8>> {
    let bytes = transact(transport, SETTINGS_COMMAND, &[])?;
    if bytes.len() < 19 {
        return Err(format!("MCHOSE 设置数据不足：{} 字节", bytes.len()));
    }
    Ok(bytes)
}

fn write_settings(transport: &mut WinTransport, bytes: &[u8]) -> Result<()> {
    // V3 writes are command-only; the source driver waits before issuing the
    // explicit settings read-back that confirms application.
    let request = build_command(WRITE_SETTINGS_COMMAND, bytes);
    transport.write(&request, Duration::from_millis(500))?;
    std::thread::sleep(WRITE_SETTLE);
    Ok(())
}

fn confirmed_settings_write(
    transport: &mut WinTransport,
    original: &[u8],
    desired: &[u8],
) -> Result<Vec<u8>> {
    let attempted: Result<Vec<u8>> = (|| {
        write_settings(transport, desired)?;
        let actual = settings(transport)?;
        if actual != desired {
            return Err("设置读回与目标值不符".into());
        }
        Ok(actual)
    })();
    match attempted {
        Ok(actual) => Ok(actual),
        Err(cause) => {
            let restored = write_settings(transport, original)
                .and_then(|_| settings(transport))
                .and_then(|bytes| {
                    if bytes == original {
                        Ok(())
                    } else {
                        Err("恢复值读回不符".into())
                    }
                });
            match restored {
                Ok(()) => Err(format!("MCHOSE 设置失败：{cause}；原设置已恢复并读回确认")),
                Err(rollback) => Err(format!(
                    "MCHOSE 设置失败：{cause}；回滚未确认：{rollback}；请检查设备设置"
                )),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ButtonAssignment {
    kind: u8,
    value: Vec<u8>,
}

const BUTTON_NAMES: [&str; 6] = ["左键", "右键", "中键", "前进", "后退", "DPI"];

// Read header: profile, axis, count, active, separate-Y.
// Write header: profile, axis, separate-Y, count, active.
fn dpi_write_payload(read: &[u8]) -> Result<Vec<u8>> {
    if read.len() < 17 || !(1..=6).contains(&read[2]) || read[3] >= read[2] || read[4] > 1 {
        return Err("MCHOSE DPI 档位结构未知，未执行写入".into());
    }
    let mut write = read.to_vec();
    write[2..5].copy_from_slice(&[read[4], read[2], read[3]]);
    Ok(write)
}

fn decode_buttons(payload: &[u8]) -> Result<Vec<ButtonAssignment>> {
    let mut buttons = Vec::with_capacity(6);
    let mut offset = 0;
    for _ in 0..6 {
        let kind = *payload.get(offset).ok_or("MCHOSE 键位数据不完整")?;
        let width = match kind {
            0x01 | 0x22 => 3,
            0x23 | 0x24 => 7,
            _ => 2,
        };
        if offset + 1 + width > payload.len() {
            return Err("MCHOSE 键位数据不完整".into());
        }
        let mut value = payload[offset + 1..offset + 1 + width].to_vec();
        if !matches!(kind, 0x13 | 0x16) {
            value.reverse();
        }
        buttons.push(ButtonAssignment { kind, value });
        offset += 1 + width;
    }
    Ok(buttons)
}

fn encode_buttons(profile: u8, buttons: &[ButtonAssignment]) -> Result<Vec<u8>> {
    if buttons.len() != 6 {
        return Err("MCHOSE 键位表长度不匹配".into());
    }
    let mut out = vec![profile, 0, 6];
    for button in buttons {
        let width = match button.kind {
            0x01 | 0x22 => 3,
            0x23 | 0x24 => 7,
            _ => 2,
        };
        if button.value.len() != width {
            return Err("MCHOSE 键位动作长度不匹配".into());
        }
        out.push(button.kind);
        let mut value = button.value.clone();
        if !matches!(button.kind, 0x13 | 0x16) {
            value.reverse();
        }
        out.extend(value);
    }
    Ok(out)
}

fn assignment_from_action(action: &BindingAction, button: usize) -> Result<ButtonAssignment> {
    let assignment = match action {
        BindingAction::Disabled => ButtonAssignment {
            kind: 0xfe,
            value: vec![0, 0],
        },
        BindingAction::DpiCycle => ButtonAssignment {
            kind: 0x01,
            value: vec![0, 0, 0],
        },
        BindingAction::Mouse(1) => ButtonAssignment {
            kind: 0,
            value: vec![0, 1],
        },
        BindingAction::Mouse(2) => ButtonAssignment {
            kind: 0,
            value: vec![0, 2],
        },
        BindingAction::Mouse(3) => ButtonAssignment {
            kind: 0,
            value: vec![0, 4],
        },
        BindingAction::Mouse(4) => ButtonAssignment {
            kind: 0,
            value: vec![0, 8],
        },
        BindingAction::Mouse(5) => ButtonAssignment {
            kind: 0,
            value: vec![0, 0x10],
        },
        BindingAction::Unknown(raw) if raw.len() >= 2 => ButtonAssignment {
            kind: raw[0],
            value: raw[1..].to_vec(),
        },
        _ => return Err("此预览支持禁用、鼠标键、DPI 循环或原始未知键位原样写回".into()),
    };
    let is_default = matches!(action, BindingAction::Unknown(_));
    if !is_default && (button >= 6 || (assignment.kind == 0 && assignment.value == [0, 0])) {
        return Err("MCHOSE 键位动作无效".into());
    }
    Ok(assignment)
}

fn action_from_assignment(button: usize, assignment: &ButtonAssignment) -> BindingAction {
    let kind = assignment.kind;
    let value = assignment.value.as_slice();
    match (kind, value) {
        (0xfe, [0, 0]) => BindingAction::Disabled,
        (0x01, [0, 0, 0]) => BindingAction::DpiCycle,
        (0, [0, 1]) => BindingAction::Mouse(1),
        (0, [0, 2]) => BindingAction::Mouse(2),
        (0, [0, 4]) => BindingAction::Mouse(3),
        (0, [0, 8]) => BindingAction::Mouse(4),
        (0, [0, 0x10]) => BindingAction::Mouse(5),
        (0, [0, 0]) if button == 5 => BindingAction::Unknown(vec![kind, value[0], value[1]]),
        _ => {
            let mut raw = vec![kind];
            raw.extend_from_slice(value);
            BindingAction::Unknown(raw)
        }
    }
}

fn read_controls(transport: &mut WinTransport) -> Result<ControlState> {
    let cfg = settings(transport)?;
    let profile = cfg[0];
    let raw = transact(transport, BUTTONS_COMMAND, &[profile, 0, 6])?;
    let buttons = decode_buttons(&raw)?;
    Ok(ControlState {
        bindings: buttons
            .iter()
            .enumerate()
            .map(|(i, b)| Binding {
                control: i as u16,
                layer: 0,
                label: BUTTON_NAMES[i].into(),
                action: action_from_assignment(i, b),
                editable: true,
            })
            .collect(),
        binding_types: vec!["mouse".into(), "dpi".into(), "disabled".into()],
        notice:
            "仅展示 M HUB 键位表；本预览写入只支持禁用、鼠标键和 DPI 循环，其他现有编码会保留。"
                .into(),
        ..ControlState::default()
    })
}

pub fn read_controls_state() -> Result<ControlState> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = matches.next() else {
        return Ok(ControlState {
            notice: "未找到精确的 MCHOSE A7 V3 Ultra+ 接收器集合".into(),
            ..ControlState::default()
        });
    };
    if matches.next().is_some() {
        return Err("找到多个 MCHOSE 接收器集合，已停止读取".into());
    }
    let mut transport = WinTransport::open(&interface.path)?;
    let info = transact(&mut transport, INFO_COMMAND, &[])?;
    let info: [u8; 16] = info.try_into().map_err(|_| "MCHOSE 身份数据长度不匹配")?;
    verify_identity(&info, &interface.product)?;
    read_controls(&mut transport)
}

pub fn apply_control_action(action: &ControlAction) -> Result<ControlState> {
    let ControlAction::Bind {
        control,
        layer,
        action,
    } = action
    else {
        return Err("MCHOSE A7 V3 Ultra+ 此预览仅支持单键重映射".into());
    };
    if *layer != 0 || *control >= 6 {
        return Err("MCHOSE 键位编号无效".into());
    }
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = matches.next() else {
        return Err("未找到精确的 MCHOSE 接收器集合".into());
    };
    if matches.next().is_some() {
        return Err("找到多个 MCHOSE 接收器集合，已停止写入".into());
    }
    let mut transport = WinTransport::open(&interface.path)?;
    let info = transact(&mut transport, INFO_COMMAND, &[])?;
    let info: [u8; 16] = info.try_into().map_err(|_| "MCHOSE 身份数据长度不匹配")?;
    verify_identity(&info, &interface.product)?;
    let cfg = settings(&mut transport)?;
    let before = transact(&mut transport, BUTTONS_COMMAND, &[cfg[0], 0, 6])?;
    let mut buttons = decode_buttons(&before)?;
    buttons[*control as usize] = assignment_from_action(action, *control as usize)?;
    if !buttons
        .iter()
        .enumerate()
        .any(|(index, button)| action_from_assignment(index, button) == BindingAction::Mouse(1))
    {
        return Err("至少保留一个左键；未执行改键".into());
    }
    let desired = encode_buttons(cfg[0], &buttons)?;
    let write = build_command(WRITE_BUTTONS_COMMAND, &desired);
    let attempted = transport
        .write(&write, Duration::from_millis(500))
        .map(|_| {
            std::thread::sleep(WRITE_SETTLE);
        })
        .and_then(|_| transact(&mut transport, BUTTONS_COMMAND, &[cfg[0], 0, 6]))
        .and_then(|confirmed| {
            if decode_buttons(&confirmed)
                .map(|got| got == buttons)
                .unwrap_or(false)
            {
                Ok(())
            } else {
                Err("键位读回与目标值不符".into())
            }
        });
    if let Err(cause) = attempted {
        let rollback = encode_buttons(cfg[0], &decode_buttons(&before)?)?;
        let restored = transport
            .write(
                &build_command(WRITE_BUTTONS_COMMAND, &rollback),
                Duration::from_millis(500),
            )
            .map(|_| {
                std::thread::sleep(WRITE_SETTLE);
            })
            .and_then(|_| transact(&mut transport, BUTTONS_COMMAND, &[cfg[0], 0, 6]))
            .and_then(|actual| {
                if actual == before {
                    Ok(())
                } else {
                    Err("恢复值读回不符".into())
                }
            });
        if let Err(rollback_error) = restored {
            return Err(format!(
                "MCHOSE 键位写入失败：{cause}；回滚未确认：{rollback_error}；请检查设备设置"
            ));
        }
        return Err(format!(
            "MCHOSE 键位写入失败：{cause}；原键位已恢复并读回确认"
        ));
    }
    read_controls(&mut transport)
}

pub fn apply_action(action: Action) -> Result<State> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = matches.next() else {
        return Err("未找到精确的 MCHOSE A7 V3 Ultra+ 接收器集合".into());
    };
    if matches.next().is_some() {
        return Err("找到多个 MCHOSE 接收器集合，已停止写入".into());
    }
    let mut transport = WinTransport::open(&interface.path)?;
    let identity = transact(&mut transport, INFO_COMMAND, &[])?;
    if identity.len() != 16 {
        return Err("MCHOSE 型号身份数据长度不匹配；拒绝写入".into());
    }
    let identity: [u8; 16] = identity.try_into().map_err(|_| "MCHOSE 身份数据不完整")?;
    verify_identity(&identity, &interface.product)?;
    let original = settings(&mut transport)?;
    let mut desired = original.clone();
    match action {
        Action::Rate(rate) if [125, 500, 1000, 2000, 4000, 8000].contains(&rate) => {
            // V3's radio rate index is stored in the high nibble of byte 2.
            let option_index = [125, 500, 1000, 2000, 4000, 8000]
                .iter()
                .position(|v| *v == rate)
                .unwrap() as u8;
            let wire_index = if option_index == 0 {
                0
            } else {
                option_index + 1
            };
            desired[2] = (desired[2] & 0x0f) | (wire_index << 4);
        }
        Action::Dpi(dpi) if (50..=MAX_DPI).contains(&dpi) && dpi.is_multiple_of(DPI_STEP) => {
            let profile = original[0];
            let settings_dpi_index = original[1] & 0x0f;
            let current = transact(&mut transport, DPI_COMMAND, &[profile, 0])?;
            if current.len() < 17 {
                return Err("MCHOSE DPI 数据不足；拒绝写入".into());
            }
            dpi_write_payload(&current)?;
            if current[0] != profile
                || current[1] != 0
                || current[3] as usize != settings_dpi_index as usize
            {
                return Err(
                    "MCHOSE DPI profile/active-stage readback disagrees with fresh settings".into(),
                );
            }
            let mut next = current.clone();
            let at = 5 + settings_dpi_index as usize * 2;
            next[at..at + 2].copy_from_slice(&dpi.to_le_bytes());
            let request = dpi_write_payload(&next)?;
            let write = build_command(WRITE_DPI_COMMAND, &request);
            let attempted = transport
                .write(&write, Duration::from_millis(500))
                .map(|_| {
                    std::thread::sleep(WRITE_SETTLE);
                })
                .and_then(|_| transact(&mut transport, DPI_COMMAND, &[profile, 0]))
                .and_then(|confirmed| {
                    if confirmed == next {
                        Ok(())
                    } else {
                        Err("DPI 读回与目标值不符".into())
                    }
                });
            if let Err(cause) = attempted {
                let rollback = build_command(WRITE_DPI_COMMAND, &dpi_write_payload(&current)?);
                let restored = transport
                    .write(&rollback, Duration::from_millis(500))
                    .map(|_| {
                        std::thread::sleep(WRITE_SETTLE);
                    })
                    .and_then(|_| transact(&mut transport, DPI_COMMAND, &[profile, 0]))
                    .and_then(|actual| {
                        if actual == current {
                            Ok(())
                        } else {
                            Err("恢复值读回不符".into())
                        }
                    });
                if let Err(rollback_error) = restored {
                    return Err(format!("MCHOSE DPI 写入失败：{cause}；回滚未确认：{rollback_error}；请检查设备设置"));
                }
                return Err(format!(
                    "MCHOSE DPI 写入失败：{cause}；原 DPI 已恢复并读回确认"
                ));
            }
            return read_state();
        }
        Action::Rate(_) => {
            return Err("A7 V3 Ultra+ 无线回报率仅支持 125、500、1000、2000、4000、8000 Hz".into())
        }
        Action::Dpi(_) => return Err("A7 V3 Ultra+ DPI 必须为 50–50000 且是 50 的倍数".into()),
        _ => return Err("此预览仅允许 DPI 与无线回报率设置".into()),
    }
    confirmed_settings_write(&mut transport, &original, &desired)?;
    read_state()
}

fn matching_interface(i: &peripheral_desk::hid::Interface) -> bool {
    i.vid == VID
        && i.pid == RECEIVER_PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.product == PRODUCT_NAME
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

pub fn read_state() -> Result<peripheral_desk::model::State> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut matches = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = matches.next() else {
        return Ok(peripheral_desk::model::State {
            key: DEVICE_KEY.into(),
            error: "未找到精确的 MCHOSE A7 V3 Ultra+ 8K 接收器 HID 集合".into(),
            settings_error: "仅提供未实测的只读身份与电量预览；无设置写入。".into(),
            updated: now(),
            ..Default::default()
        });
    };
    if matches.next().is_some() {
        return Err("找到多个匹配的 MCHOSE 接收器集合；为避免读取错误设备已停止".into());
    }

    let mut transport = WinTransport::open(&interface.path)?;
    let request = request();
    let deadline = Instant::now() + Duration::from_millis(1500);
    transport.flush()?;
    transport.write(
        &request,
        deadline
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(500)),
    )?;
    let response = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("MCHOSE 0x0900 身份/电量查询超时".into());
        }
        let wire = transport.read(REPORT_LEN, remaining)?;
        if wire.is_empty() {
            continue;
        }
        if wire.len() == REPORT_LEN
            && wire[0] == REPORT_ID
            && wire[1] == request[1]
            && wire[4..6] == request[4..6]
        {
            break wire;
        }
    };
    let data = reply_info(&response, &request)?;
    let (percent, charging, connected) = verify_identity(&data, &interface.product)?;
    let settings_result = transact(&mut transport, SETTINGS_COMMAND, &[]);
    let mut state = peripheral_desk::model::State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: connected,
        percent: connected.then_some(percent),
        charging: connected.then_some(charging),
        wired: false,
        updated: now(),
        identity: "MCHOSE A7 V3 Ultra+ · USB 3837:1018 receiver · identity 3837:4026".into(),
        error: if connected {
            String::new()
        } else {
            "MCHOSE 接收器已响应，但报告鼠标未连接；未显示电量".into()
        },
        settings_error:
            "写入按上游 M HUB 命令实现并逐项读回；此预览未经本机硬件验证。该型号无可控鼠标灯光。"
                .into(),
        ..Default::default()
    };
    if let Ok(raw) = settings_result {
        if raw.len() >= 19 {
            let profile = raw[0];
            let rate_index = (raw[2] >> 4) as usize;
            let rates = [125, 500, 1000, 2000, 4000, 8000];
            state.rate = match rate_index {
                0 => Some(rates[0]),
                2 => Some(rates[1]),
                3 => Some(rates[2]),
                4 => Some(rates[3]),
                5 => Some(rates[4]),
                6 => Some(rates[5]),
                _ => None,
            };
            let dpi = transact(&mut transport, DPI_COMMAND, &[profile, 0]);
            if let Ok(dpi) = dpi {
                if dpi.len() >= 17 && dpi[0] == profile && dpi[1] == 0 {
                    let count = usize::from(dpi[2]).min(6);
                    let active = usize::from(dpi[3]);
                    let mut stages = Vec::new();
                    for i in 0..count {
                        let at = 5 + i * 2;
                        let value = u16::from_le_bytes([dpi[at], dpi[at + 1]]);
                        if value > 0 && value <= MAX_DPI {
                            stages.push(value);
                        }
                    }
                    if active < count && active < stages.len() {
                        state.dpi = Some(stages[active]);
                    }
                }
            }
        }
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    #[test]
    fn dpi_write_reorders_header_and_preserves_untouched_data() {
        let read = [
            2, 0, 6, 3, 1, 0x20, 3, 0x40, 6, 0x20, 3, 0x40, 6, 0x20, 3, 0x40, 6,
        ];
        let write = super::dpi_write_payload(&read).unwrap();
        assert_eq!(&write[..5], &[2, 0, 1, 6, 3]);
        assert_eq!(&write[5..], &read[5..]);
        let mut bad = read;
        bad[3] = 6;
        assert!(super::dpi_write_payload(&bad).is_err());
    }
    use super::*;

    fn captured_identity_reply(req: &[u8; REPORT_LEN]) -> [u8; REPORT_LEN] {
        // Identity/power payload from the fixed upstream A7 V3 Ultra+ receiver capture.
        let payload = [
            0x37, 0x38, 0x26, 0x40, 4, 0, 0, 0, 0, 0x10, 2, 1, 0x55, 0, 8, 0xe4,
        ];
        let mut wire = [0; REPORT_LEN];
        wire[0] = REPORT_ID;
        let body = &mut wire[1..];
        body[0] = 1;
        body[1] = 1;
        body[2] = payload.len() as u8;
        body[3..5].copy_from_slice(&INFO_COMMAND.to_le_bytes());
        body[5] = req[6];
        body[6] = req[7];
        body[7..7 + payload.len()].copy_from_slice(&payload);
        body[7 + payload.len()] = checksum(&body[1..7 + payload.len()]);
        wire
    }

    #[test]
    fn builds_only_the_zero_payload_identity_read() {
        let wire = request();
        assert_eq!(wire[0], REPORT_ID);
        assert_eq!(&wire[1..8], &[1, 1, 0, 0, 9, 0, 0]);
        assert_eq!(wire[8], checksum(&wire[2..8]));
    }

    #[test]
    fn builds_settings_commands_without_touching_unknown_payload_bytes() {
        let bytes: Vec<u8> = (0..19).collect();
        let wire = build_command(WRITE_SETTINGS_COMMAND, &bytes);
        assert_eq!(wire[1 + 3..1 + 5], WRITE_SETTINGS_COMMAND.to_le_bytes());
        assert_eq!(wire[1 + 7..1 + 7 + bytes.len()], bytes);
        assert_eq!(
            wire[1 + 7 + bytes.len()],
            checksum(&wire[2..1 + 7 + bytes.len()])
        );
    }

    #[test]
    fn button_codec_preserves_unknown_assignments_and_maps_only_known_actions() {
        let payload = [
            0, 1, 0, 0xfe, 0, 0, 1, 0, 0, 0, 0x22, 0x11, 0x22, 0x33, 0x23, 0, 1, 2, 3, 4, 5, 6,
            0x01, 0, 0, 0,
        ];
        let decoded = decode_buttons(&payload).unwrap();
        assert_eq!(
            decoded[0],
            ButtonAssignment {
                kind: 0,
                value: vec![0, 1]
            }
        );
        assert_eq!(
            action_from_assignment(0, &decoded[0]),
            BindingAction::Mouse(1)
        );
        assert_eq!(encode_buttons(3, &decoded).unwrap()[3..], payload);
        let unknown = action_from_assignment(3, &decoded[3]);
        assert_eq!(assignment_from_action(&unknown, 3).unwrap(), decoded[3]);
    }

    #[test]
    fn validates_captured_identity_and_rejects_corrupt_or_mismatched_reply() {
        let req = request();
        let mut reply = captured_identity_reply(&req);
        let data = reply_info(&reply, &req).unwrap();
        assert_eq!(
            verify_identity(&data, PRODUCT_NAME).unwrap(),
            (0x55, true, true)
        );
        assert!(verify_identity(&data, "MCHOSE A5 V3 Ultra+").is_err());

        reply[1 + 20] ^= 1;
        assert!(reply_info(&reply, &req).is_err());
        let mut reply = captured_identity_reply(&req);
        reply[1 + 3] ^= 1;
        assert!(reply_info(&reply, &req).is_err());
    }

    #[test]
    fn refuses_unknown_charge_and_out_of_range_battery() {
        let mut data = [0; 16];
        data[..4].copy_from_slice(&[0x37, 0x38, 0x26, 0x40]);
        assert!(verify_identity(&data, PRODUCT_NAME).is_ok());
        data[11] = 2;
        assert!(verify_identity(&data, PRODUCT_NAME).is_err());
        data[11] = 0;
        data[12] = 101;
        assert!(verify_identity(&data, PRODUCT_NAME).is_err());
    }

    #[test]
    fn disconnected_and_unknown_connection_states_are_not_presented_as_fresh() {
        let req = request();
        let mut reply = captured_identity_reply(&req);
        reply[1 + 7 + 10] = 0;
        let checksum_at = 1 + 7 + 16;
        reply[checksum_at] = checksum(&reply[2..checksum_at]);
        let data = reply_info(&reply, &req).unwrap();
        assert_eq!(
            verify_identity(&data, PRODUCT_NAME).unwrap(),
            (0x55, true, false)
        );

        reply[1 + 7 + 10] = 3;
        reply[checksum_at] = checksum(&reply[2..checksum_at]);
        let data = reply_info(&reply, &req).unwrap();
        assert!(verify_identity(&data, PRODUCT_NAME).is_err());
    }

    #[test]
    fn rejects_the_upstream_unreachable_mouse_marker() {
        let req = request();
        let mut wire = [0; REPORT_LEN];
        wire[0] = REPORT_ID;
        let body = &mut wire[1..];
        body[..7].copy_from_slice(&[1, 1, 1, 0, 9, 0, 0]);
        body[7] = 0xff;
        body[8] = checksum(&body[1..8]);
        assert!(reply_info(&wire, &req).unwrap_err().contains("不可达"));
    }

    #[test]
    fn only_exact_receiver_collection_and_product_name_match() {
        let mut interface = peripheral_desk::hid::Interface {
            path: r"\\?\hid#vid_3837&pid_1018".into(),
            vid: VID,
            pid: RECEIVER_PID,
            usage_page: USAGE_PAGE,
            usage: USAGE,
            feature_len: 0,
            input_len: REPORT_LEN as u16,
            output_len: REPORT_LEN as u16,
            product: PRODUCT_NAME.into(),
        };
        assert!(matching_interface(&interface));
        interface.product.push(' ');
        assert!(!matching_interface(&interface));
        interface.product = PRODUCT_NAME.into();
        interface.pid = 0x1014;
        assert!(!matching_interface(&interface));
        interface.pid = RECEIVER_PID;
        interface.output_len = 63;
        assert!(!matching_interface(&interface));
    }
}
