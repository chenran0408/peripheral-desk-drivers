//! VGN battery, DPI, polling-rate, and binding codecs. F2 Master+ protocol facts
//! are sourced from OpenMouse `mouse-protocol` commit
//! `bac031567b28f2a55c616fc525b03f2d5fa452cc`; F1 Pro and F2 Pro Max PID and
//! battery facts are sourced from `quiet-battery` commit
//! `d6eb4bdade47dccd7cb402e16ac50a6f351fcac0`. See the per-model docs.
use peripheral_desk::controls::{Binding, BindingAction, ControlState};
use peripheral_desk::model::State;
pub use peripheral_desk::{controls::ControlAction, model::Action};
use std::time::{SystemTime, UNIX_EPOCH};

pub type Result<T> = std::result::Result<T, String>;

pub const REPORT_ID: u8 = 0x08;
pub const PAYLOAD_LEN: usize = 16;
pub const VID: u16 = 0x3554;
pub const DEVICE_KEY: &str = F2_MASTER_PLUS.key;
pub const RECEIVER_PID: u16 = F2_MASTER_PLUS.receiver_pid;
pub const WIRED_PID: u16 = F2_MASTER_PLUS.wired_pid;
const BATTERY: u8 = 0x04;
const FIRMWARE: u8 = 0x12;
const READ: u8 = 0x08;
const WRITE: u8 = 0x07;
const PROFILE_HEAD_LEN: usize = 0x2c;
const KEY_TABLE_ADDRESS: u16 = 0x0060;
const KEY_RECORD_LEN: usize = 4;
const KEY_COUNT: usize = 6;

pub const ADDRESS_POLLING_RATE: u16 = 0x0000;
pub const ADDRESS_DPI_STAGE_COUNT: u16 = 0x0002;
pub const ADDRESS_ACTIVE_DPI_STAGE: u16 = 0x0004;
pub const ADDRESS_DPI_STAGES: u16 = 0x000c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceSpec {
    pub key: &'static str,
    pub name: &'static str,
    pub receiver_pid: u16,
    pub wired_pid: u16,
    pub reads_firmware: bool,
    pub required_usage: Option<u16>,
    pub rejects_battery_echo: bool,
}

pub const F2_MASTER_PLUS: DeviceSpec = DeviceSpec {
    key: "vgn-f2-master-plus",
    name: "VGN Dragonfly F2 Master+",
    receiver_pid: 0xfb56,
    wired_pid: 0xfb57,
    reads_firmware: true,
    required_usage: None,
    rejects_battery_echo: false,
};
pub const F2_PRO_MAX: DeviceSpec = DeviceSpec {
    key: "vgn-f2-pro-max",
    name: "VGN Dragonfly F2 Pro Max",
    receiver_pid: 0xfb3e,
    wired_pid: 0xfb3d,
    reads_firmware: false,
    required_usage: Some(2),
    rejects_battery_echo: true,
};
pub const F1_PRO: DeviceSpec = DeviceSpec {
    key: "vgn-f1-pro",
    name: "VGN F1 Pro",
    receiver_pid: 0xf503,
    wired_pid: 0xf502,
    reads_firmware: false,
    required_usage: Some(2),
    rejects_battery_echo: true,
};

/// A serialized report-08 transaction. The Windows transport adds/removes the
/// report-ID byte and must cancel and drain timed-out overlapped I/O.
pub trait Transport {
    fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>>;
}

mod transport;
pub use transport::{
    apply_vgn_controls, apply_vgn_setting, read_vgn, read_vgn_controls, read_vgn_for,
};

pub fn checksum(payload: &[u8; PAYLOAD_LEN]) -> u8 {
    let sum = payload[..15]
        .iter()
        .fold(REPORT_ID as u16, |sum, byte| sum + u16::from(*byte));
    (0x55u16.wrapping_sub(sum & 0xff)) as u8
}

pub fn checksum_valid(payload: &[u8]) -> bool {
    payload.len() == PAYLOAD_LEN
        && (payload
            .iter()
            .fold(REPORT_ID as u16, |sum, byte| sum + u16::from(*byte))
            & 0xff)
            == 0x55
}

fn simple_request(command: u8) -> [u8; PAYLOAD_LEN] {
    let mut payload = [0; PAYLOAD_LEN];
    payload[0] = command;
    payload[15] = checksum(&payload);
    payload
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
    pub voltage_mv: u16,
}

pub fn parse_battery(response: &[u8]) -> Result<Battery> {
    if !checksum_valid(response) {
        return Err("VGN 电量回复长度或校验和不匹配".into());
    }
    if response[0] != BATTERY || response[1] != 0 {
        return Err("VGN 电量回复命令或状态不匹配".into());
    }
    let percent = response[5];
    if percent > 100 {
        return Err(format!("VGN 电量超出 0–100：{percent}"));
    }
    let charging = match response[6] {
        0 => false,
        1 => true,
        value => return Err(format!("VGN 返回未知充电状态 0x{value:02X}")),
    };
    Ok(Battery {
        percent,
        charging,
        voltage_mv: u16::from_be_bytes([response[7], response[8]]),
    })
}

pub fn parse_firmware(response: &[u8]) -> Option<String> {
    if !checksum_valid(response) || response[0] != FIRMWARE || response[1] != 0 {
        return None;
    }
    Some(format!("v{}.{:02x}", response[5], response[6]))
}

pub fn build_read(address: u16, length: usize) -> Result<[u8; PAYLOAD_LEN]> {
    if !(1..=10).contains(&length) {
        return Err("VGN 读取长度必须为 1–10 字节".into());
    }
    let mut payload = simple_request(READ);
    payload[2..4].copy_from_slice(&address.to_be_bytes());
    payload[4] = length as u8;
    payload[15] = checksum(&payload);
    Ok(payload)
}

pub fn build_write(address: u16, data: &[u8]) -> Result<[u8; PAYLOAD_LEN]> {
    if data.is_empty() || data.len() > 10 {
        return Err("VGN 写入长度必须为 1–10 字节".into());
    }
    let mut payload = simple_request(WRITE);
    payload[2..4].copy_from_slice(&address.to_be_bytes());
    payload[4] = data.len() as u8;
    payload[5..5 + data.len()].copy_from_slice(data);
    payload[15] = checksum(&payload);
    Ok(payload)
}

pub fn parse_read(response: &[u8], address: u16, length: usize) -> Result<Vec<u8>> {
    if !checksum_valid(response)
        || response[0] != READ
        || response[1] != 0
        || response[2..4] != address.to_be_bytes()
        || usize::from(response[4]) != length
        || length > 10
    {
        return Err(format!("VGN flash 读取 0x{address:04X} 回复不匹配"));
    }
    Ok(response[5..5 + length].to_vec())
}

pub fn decode_polling_rate(raw: u8) -> Option<u16> {
    match raw {
        8 => Some(125),
        4 => Some(250),
        2 => Some(500),
        1 => Some(1000),
        16 => Some(2000),
        32 => Some(4000),
        64 => Some(8000),
        _ => None,
    }
}

pub fn encode_polling_rate(rate: u16) -> Result<u8> {
    match rate {
        125 => Ok(8),
        250 => Ok(4),
        500 => Ok(2),
        1000 => Ok(1),
        2000 => Ok(16),
        4000 => Ok(32),
        8000 => Ok(64),
        _ => Err("VGN polling rate must be 125, 250, 500, 1000, 2000, 4000, or 8000 Hz".into()),
    }
}

pub fn decode_dpi(stage: &[u8]) -> Option<u16> {
    if stage.len() < 4
        || stage[0] != stage[1]
        || (stage[..4].iter().map(|v| u16::from(*v)).sum::<u16>() & 0xff) != 0x55
    {
        return None;
    }
    let steps = (u16::from((stage[2] >> 2) & 3) << 8) | u16::from(stage[0]);
    let dpi = (steps + 1) * 50;
    (50..=26_000).contains(&dpi).then_some(dpi)
}

pub fn encode_dpi_preserving(stage: &[u8], dpi: u16) -> Result<[u8; 4]> {
    if stage.len() != 4 || decode_dpi(stage).is_none() {
        return Err("VGN DPI stage is invalid; keeping it read-only".into());
    }
    if !(50..=26_000).contains(&dpi) || !dpi.is_multiple_of(50) {
        return Err("VGN DPI must be 50–26,000 in 50 DPI steps".into());
    }
    let encoded = dpi / 50 - 1;
    let low = encoded as u8;
    let high = ((encoded >> 8) & 3) as u8;
    let flags = (stage[2] & !0xcc) | (high << 2) | (high << 6);
    let mut result = [low, low, flags, 0];
    result[3] = (0x55u16.wrapping_sub(u16::from(low) * 2 + u16::from(flags)) & 0xff) as u8;
    Ok(result)
}

fn scalar(profile: &[u8], address: usize) -> Result<u8> {
    let value = *profile.get(address).ok_or("VGN profile is incomplete")?;
    let parity = *profile
        .get(address + 1)
        .ok_or("VGN profile is incomplete")?;
    if (u16::from(value) + u16::from(parity)) & 0xff != 0x55 {
        return Err(format!(
            "VGN profile scalar 0x{address:02X} has an invalid parity byte"
        ));
    }
    Ok(value)
}

fn set_profile_state(state: &mut State, profile: &[u8]) -> Result<()> {
    let count = scalar(profile, ADDRESS_DPI_STAGE_COUNT as usize)?;
    if !(1..=8).contains(&count) {
        return Err(format!(
            "VGN profile reports unsupported DPI stage count {count}"
        ));
    }
    let active = scalar(profile, ADDRESS_ACTIVE_DPI_STAGE as usize)?;
    if active >= count {
        return Err(format!(
            "VGN profile reports invalid active DPI stage {active}"
        ));
    }
    let mut stages = Vec::with_capacity(usize::from(count));
    for stage in 0..usize::from(count) {
        let offset = ADDRESS_DPI_STAGES as usize + stage * 4;
        let raw = profile
            .get(offset..offset + 4)
            .ok_or("VGN DPI table is incomplete")?;
        let dpi = decode_dpi(raw).ok_or_else(|| format!("VGN DPI stage {stage} is invalid"))?;
        stages.push(dpi);
    }
    let rate = decode_polling_rate(scalar(profile, ADDRESS_POLLING_RATE as usize)?)
        .ok_or("VGN profile has an unknown polling-rate code")?;
    state.dpi = Some(stages[usize::from(active)]);
    state.dpi_stages = stages;
    state.active_stage = Some(active);
    state.rate = Some(rate);
    Ok(())
}

pub fn refresh_profile_state<T: Transport>(transport: &mut T, state: &mut State) -> Result<()> {
    let profile = read_profile(transport)?;
    set_profile_state(state, &profile)
}

pub fn read_profile<T: Transport>(transport: &mut T) -> Result<Vec<u8>> {
    // DPI/rate live entirely in this header; omit two unrelated tail reads.
    let mut profile = vec![0; PROFILE_HEAD_LEN];
    let mut address = 0;
    while address < PROFILE_HEAD_LEN {
        let length = (PROFILE_HEAD_LEN - address).min(10);
        let response = transport.exchange(&build_read(address as u16, length)?)?;
        let bytes = parse_read(&response, address as u16, length)?;
        profile[address..address + length].copy_from_slice(&bytes);
        address += length;
    }
    Ok(profile)
}

fn write_and_confirm<T: Transport>(
    transport: &mut T,
    address: u16,
    before: &[u8],
    after: &[u8],
) -> Result<()> {
    if before.len() != after.len() || before.is_empty() || after.len() > 10 {
        return Err("VGN setting block length mismatch".into());
    }
    let result = (|| {
        let response = transport.exchange(&build_write(address, after)?)?;
        if !checksum_valid(&response) || response[0] != WRITE || response[1] != 0 {
            return Err(format!(
                "VGN write acknowledgement failed at 0x{address:04X}"
            ));
        }
        let response = transport.exchange(&build_read(address, after.len())?)?;
        let actual = parse_read(&response, address, after.len())?;
        if actual != after {
            return Err(format!("VGN read-back mismatch at 0x{address:04X}"));
        }
        Ok(())
    })();
    if let Err(error) = result {
        let restore = (|| {
            let response = transport.exchange(&build_write(address, before)?)?;
            if !checksum_valid(&response) || response[0] != WRITE || response[1] != 0 {
                return Err(format!(
                    "rollback acknowledgement failed at 0x{address:04X}"
                ));
            }
            let response = transport.exchange(&build_read(address, before.len())?)?;
            let actual = parse_read(&response, address, before.len())?;
            if actual != before {
                return Err(format!("rollback read-back mismatch at 0x{address:04X}"));
            }
            Ok(())
        })();
        return Err(match restore {
            Ok(()) => format!("VGN write failed: {error}; original bytes restored and confirmed"),
            Err(rollback) => {
                format!("VGN write failed: {error}; rollback could not be confirmed: {rollback}")
            }
        });
    }
    Ok(())
}

pub fn apply_setting<T: Transport>(transport: &mut T, action: Action) -> Result<()> {
    let profile = read_profile(transport)?;
    let mut baseline = State::default();
    set_profile_state(&mut baseline, &profile)?;
    match action {
        Action::Dpi(dpi) => {
            let active = scalar(&profile, ADDRESS_ACTIVE_DPI_STAGE as usize)?;
            let address = ADDRESS_DPI_STAGES + u16::from(active) * 4;
            let offset = address as usize;
            let before = &profile[offset..offset + 4];
            let after = encode_dpi_preserving(before, dpi)?;
            write_and_confirm(transport, address, before, &after)
        }
        Action::Stage(stage) => {
            if usize::from(stage) >= baseline.dpi_stages.len() { return Err("DPI 档位编号超出设备范围".into()); }
            let address = ADDRESS_ACTIVE_DPI_STAGE;
            let before = &profile[address as usize..address as usize + 2];
            let after = [stage,0x55u8.wrapping_sub(stage)];
            write_and_confirm(transport,address,before,&after)
        }
        Action::Rate(rate) => {
            let before = &profile[..2];
            let value = encode_polling_rate(rate)?;
            let after = [value, (0x55u8).wrapping_sub(value)];
            write_and_confirm(transport, ADDRESS_POLLING_RATE, before, &after)
        }
        _ => Err(
            "VGN F2 Master+ preview currently supports DPI, stage selection, and polling-rate changes only".into(),
        ),
    }
}

const BUTTONS: [(u16, &str); KEY_COUNT] = [
    (0, "左键"),
    (1, "右键"),
    (2, "中键"),
    (3, "后退侧键"),
    (4, "前进侧键"),
    (5, "DPI 档位键"),
];

fn decode_binding(raw: &[u8]) -> BindingAction {
    if raw.len() != KEY_RECORD_LEN
        || (raw.iter().map(|v| u16::from(*v)).sum::<u16>() & 0xff) != 0x55
    {
        return BindingAction::Unknown(raw.to_vec());
    }
    let parameter = u16::from_be_bytes([raw[1], raw[2]]);
    match (raw[0], parameter) {
        (0, 0) => BindingAction::Disabled,
        (1, 0x0100) => BindingAction::Mouse(1),
        (1, 0x0200) => BindingAction::Mouse(2),
        (1, 0x0400) => BindingAction::Mouse(3),
        (1, 0x0800) => BindingAction::Mouse(4),
        (1, 0x1000) => BindingAction::Mouse(5),
        (2, 0x0100) => BindingAction::DpiCycle,
        _ => BindingAction::Unknown(raw.to_vec()),
    }
}

fn encode_binding(action: &BindingAction) -> Result<[u8; KEY_RECORD_LEN]> {
    let (class, parameter): (u8, u16) = match action {
        BindingAction::Disabled => (0, 0),
        BindingAction::Mouse(1) => (1, 0x0100),
        BindingAction::Mouse(2) => (1, 0x0200),
        BindingAction::Mouse(3) => (1, 0x0400),
        BindingAction::Mouse(4) => (1, 0x0800),
        BindingAction::Mouse(5) => (1, 0x1000),
        BindingAction::DpiCycle => (2, 0x0100),
        _ => {
            return Err(
                "VGN remapping preview supports mouse buttons, disable, and DPI cycle only".into(),
            );
        }
    };
    let [high, low] = parameter.to_be_bytes();
    let mut raw = [class, high, low, 0];
    raw[3] =
        (0x55u16.wrapping_sub(u16::from(class) + u16::from(high) + u16::from(low)) & 0xff) as u8;
    Ok(raw)
}

pub fn read_controls<T: Transport>(transport: &mut T) -> Result<ControlState> {
    let mut table = Vec::with_capacity(KEY_COUNT * KEY_RECORD_LEN);
    for offset in (0..KEY_COUNT * KEY_RECORD_LEN).step_by(8) {
        let length = (KEY_COUNT * KEY_RECORD_LEN - offset).min(8);
        let response =
            transport.exchange(&build_read(KEY_TABLE_ADDRESS + offset as u16, length)?)?;
        table.extend(parse_read(
            &response,
            KEY_TABLE_ADDRESS + offset as u16,
            length,
        )?);
    }
    let bindings: Vec<_> = BUTTONS
        .iter()
        .enumerate()
        .map(|(index, (control, label))| {
            let raw = &table[index * KEY_RECORD_LEN..(index + 1) * KEY_RECORD_LEN];
            let action = decode_binding(raw);
            Binding {
                control: *control,
                layer: 0,
                label: (*label).into(),
                editable: index != 0 && !matches!(action, BindingAction::Unknown(_)),
                action,
            }
        })
        .collect();
    if !bindings
        .iter()
        .any(|binding| binding.action == BindingAction::Mouse(1))
    {
        return Err(
            "VGN key table does not contain a valid left-click assignment; remapping is disabled"
                .into(),
        );
    }
    Ok(ControlState {
        bindings,
        binding_types: ["mouse", "disabled", "dpi-cycle"].map(String::from).to_vec(),
        notice: "六键映射使用 OpenMouse 记录的 Teevolution 共用表；VGN 厂商软件未有按键表抓包。未知记录保持只读，左键不可改，且至少保留一个左键功能。".into(),
        ..ControlState::default()
    })
}

pub fn apply_controls<T: Transport>(transport: &mut T, action: &ControlAction) -> Result<()> {
    let ControlAction::Bind {
        control,
        layer,
        action,
    } = action
    else {
        return Err("VGN remapping preview does not support macros or key colors".into());
    };
    if *layer != 0 || !(1..KEY_COUNT as u16).contains(control) {
        return Err("VGN remapping preview keeps the primary left button fixed and edits only standard-layer buttons".into());
    }
    let state = read_controls(transport)?;
    let target = state
        .bindings
        .iter()
        .find(|binding| binding.control == *control)
        .ok_or("VGN button index is invalid")?;
    if !target.editable {
        return Err("Unknown VGN key-table record is read-only".into());
    }
    let before: [u8; KEY_RECORD_LEN] = {
        let address = KEY_TABLE_ADDRESS + *control * KEY_RECORD_LEN as u16;
        let response = transport.exchange(&build_read(address, KEY_RECORD_LEN)?)?;
        parse_read(&response, address, KEY_RECORD_LEN)?
            .try_into()
            .map_err(|_| "VGN key record length mismatch")?
    };
    if decode_binding(&before) != target.action {
        return Err("VGN key table changed during read; no write was sent".into());
    }
    let after = encode_binding(action)?;
    if before == after {
        return Ok(());
    }
    let mut prospective = state
        .bindings
        .iter()
        .map(|b| b.action.clone())
        .collect::<Vec<_>>();
    prospective[usize::from(*control)] = action.clone();
    if !prospective.contains(&BindingAction::Mouse(1)) {
        return Err("Keep at least one button assigned to Left Click".into());
    }
    let address = KEY_TABLE_ADDRESS + *control * KEY_RECORD_LEN as u16;
    write_and_confirm(transport, address, &before, &after)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Receiver,
    Wired,
}

impl Connection {
    pub fn from_pid(pid: u16) -> Option<Self> {
        Self::from_pid_for(pid, F2_MASTER_PLUS)
    }

    pub fn from_pid_for(pid: u16, device: DeviceSpec) -> Option<Self> {
        match pid {
            pid if pid == device.receiver_pid => Some(Self::Receiver),
            pid if pid == device.wired_pid => Some(Self::Wired),
            _ => None,
        }
    }

    fn wired(self) -> bool {
        self == Self::Wired
    }

    fn pid_for(self, device: DeviceSpec) -> u16 {
        match self {
            Self::Receiver => device.receiver_pid,
            Self::Wired => device.wired_pid,
        }
    }
}

pub fn read_with<T: Transport>(transport: &mut T, connection: Connection) -> State {
    read_with_model(transport, connection, F2_MASTER_PLUS)
}

pub fn read_with_model<T: Transport>(
    transport: &mut T,
    connection: Connection,
    device: DeviceSpec,
) -> State {
    let mut state = State {
        key: device.key.into(),
        present: true,
        wired: connection.wired(),
        identity: format!(
            "{} · USB {:04X}:{:04X} · report 08{}",
            device.name,
            VID,
            connection.pid_for(device),
            if device.reads_firmware {
                " · firmware unknown"
            } else {
                ""
            }
        ),
        settings_error: if device.key == F2_MASTER_PLUS.key {
            "支持读取 DPI 与回报率，并提供 DPI、回报率和六键映射预览；映射表依赖共用协议记录且未实机验证。无灯光写入。"
        } else if device.reads_firmware {
            "仅提供只读电量、充电状态和公开固件版本；未开放设置写入。"
        } else {
            "仅提供只读电量与充电状态；该型号未核对固件读取命令，未开放设置写入。"
        }
        .into(),
        updated: now(),
        ..Default::default()
    };
    let response = match transport.exchange(&simple_request(BATTERY)) {
        Ok(response) => response,
        Err(error) => {
            state.error = error;
            return state;
        }
    };
    if device.rejects_battery_echo && response.as_slice() == simple_request(BATTERY).as_slice() {
        state.error = "VGN 电量请求收到原样回显，未返回有效电量".into();
        return state;
    }
    let battery = match parse_battery(&response) {
        Ok(battery) => battery,
        Err(error) => {
            state.error = error;
            return state;
        }
    };
    state.fresh = true;
    state.percent = Some(battery.percent);
    state.charging = Some(battery.charging);
    if !device.reads_firmware {
        return state;
    }
    match transport.exchange(&simple_request(FIRMWARE)) {
        Ok(response) => match parse_firmware(&response) {
            Some(version) => {
                state.identity = state
                    .identity
                    .replace("firmware unknown", &format!("firmware {version}"))
            }
            None => state.error = "电量已读取；VGN 固件版本回复校验或状态不匹配".into(),
        },
        Err(error) => state.error = format!("电量已读取；固件版本不可用：{error}"),
    }
    if device.key == F2_MASTER_PLUS.key {
        if let Err(error) = refresh_profile_state(transport, &mut state) {
            state.settings_error =
                format!("电量与固件已读取；VGN DPI/回报率 profile 不可用：{error}");
        }
    }
    state
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(command: u8, status: u8) -> [u8; PAYLOAD_LEN] {
        let mut wire = [0; PAYLOAD_LEN];
        wire[0] = command;
        wire[1] = status;
        wire[15] = checksum(&wire);
        wire
    }

    fn read_response(address: usize, data: &[u8]) -> [u8; PAYLOAD_LEN] {
        let mut wire = [0; PAYLOAD_LEN];
        wire[0] = READ;
        wire[2..4].copy_from_slice(&(address as u16).to_be_bytes());
        wire[4] = data.len() as u8;
        wire[5..5 + data.len()].copy_from_slice(data);
        wire[15] = checksum(&wire);
        wire
    }

    #[test]
    fn report_checksum_matches_upstream_protocol_fixture() {
        assert_eq!(
            simple_request(0x04),
            [4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x49]
        );
        let mut wire = [0x04, 0, 0, 0, 2, 95, 1, 0x10, 0x8b, 0, 0, 0, 0, 0, 0, 0x4c];
        assert_eq!(
            parse_battery(&wire).unwrap(),
            Battery {
                percent: 95,
                charging: true,
                voltage_mv: 4235
            }
        );
        wire[5] = 101;
        wire[15] = checksum(&wire);
        assert!(parse_battery(&wire).is_err());
    }

    #[test]
    fn vgn_profile_codecs_match_pinned_upstream_fixtures_and_preserve_reserved_dpi_bits() {
        assert_eq!(
            build_read(0x00a0, 10).unwrap(),
            [8, 0, 0, 0xa0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x9b]
        );
        assert_eq!(encode_polling_rate(8000).unwrap(), 64);
        assert_eq!(decode_polling_rate(16), Some(2000));
        assert_eq!(
            encode_polling_rate(6000).unwrap_err(),
            "VGN polling rate must be 125, 250, 500, 1000, 2000, 4000, or 8000 Hz"
        );
        let original = [0x1f, 0x1f, 0x33, 0x00];
        let mut valid = original;
        valid[3] =
            (0x55u16.wrapping_sub(u16::from(valid[0]) * 2 + u16::from(valid[2])) & 0xff) as u8;
        assert_eq!(decode_dpi(&valid), Some(1600));
        let changed = encode_dpi_preserving(&valid, 2000).unwrap();
        assert_eq!(changed[2] & 0x33, valid[2] & 0x33);
        assert_eq!(decode_dpi(&changed), Some(2000));
    }

    #[test]
    fn button_codec_keeps_unknown_records_opaque_and_limits_editable_actions() {
        assert_eq!(decode_binding(&[1, 1, 0, 0x53]), BindingAction::Mouse(1));
        assert_eq!(decode_binding(&[2, 1, 0, 0x52]), BindingAction::DpiCycle);
        assert_eq!(
            decode_binding(&[0x55, 0, 0, 0]),
            BindingAction::Unknown(vec![0x55, 0, 0, 0])
        );
        assert_eq!(
            encode_binding(&BindingAction::Mouse(4)).unwrap(),
            [1, 8, 0, 0x4c]
        );
        assert!(encode_binding(&BindingAction::Keyboard {
            usage: 4,
            modifiers: 0
        })
        .is_err());
    }

    #[test]
    fn rejects_wrong_opcode_status_length_checksum_and_unknown_charge_state() {
        let mut wire = response(0x04, 0);
        wire[5] = 50;
        wire[6] = 2;
        wire[15] = checksum(&wire);
        assert!(parse_battery(&wire).is_err());
        wire[6] = 0;
        wire[15] = checksum(&wire);
        assert!(parse_battery(&wire[..15]).is_err());
        wire[15] ^= 1;
        assert!(parse_battery(&wire).is_err());
        wire[15] = checksum(&wire);
        wire[0] = 0x07;
        wire[15] = checksum(&wire);
        assert!(parse_battery(&wire).is_err());
        wire[0] = 0x04;
        wire[1] = 1;
        wire[15] = checksum(&wire);
        assert!(parse_battery(&wire).is_err());
    }

    struct Fixture(std::collections::VecDeque<Result<[u8; PAYLOAD_LEN]>>);
    impl Transport for Fixture {
        fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>> {
            assert!(request[0] == BATTERY || request[0] == FIRMWARE || request[0] == READ);
            assert!(checksum_valid(request));
            let response = self.0.pop_front().unwrap()?;
            assert_eq!(request[0], response[0]);
            Ok(response.to_vec())
        }
    }

    #[test]
    fn only_the_two_declared_pids_have_connection_modes() {
        assert_eq!(
            Connection::from_pid(RECEIVER_PID),
            Some(Connection::Receiver)
        );
        assert_eq!(Connection::from_pid(WIRED_PID), Some(Connection::Wired));
        for pid in [0xf502, 0xf503, 0xfb3d, 0xfb3e, 0xfb55, 0xfb58] {
            assert_eq!(Connection::from_pid(pid), None);
        }
    }

    #[test]
    fn read_only_state_requests_battery_then_firmware_and_never_sends_write() {
        let mut battery = response(BATTERY, 0);
        battery[5] = 72;
        battery[6] = 0;
        battery[15] = checksum(&battery);
        let mut firmware = response(FIRMWARE, 0);
        firmware[5] = 1;
        firmware[6] = 0x2a;
        firmware[15] = checksum(&firmware);
        let mut profile = [0; PROFILE_HEAD_LEN];
        profile[0..2].copy_from_slice(&[1, 0x54]);
        profile[2..4].copy_from_slice(&[1, 0x54]);
        profile[4..6].copy_from_slice(&[0, 0x55]);
        profile[12..16].copy_from_slice(&[31, 31, 0, 0x17]);
        let mut replies = vec![Ok(battery), Ok(firmware)];
        for (start, end) in [(0usize, PROFILE_HEAD_LEN)] {
            let mut address = start;
            while address < end {
                let length = (end - address).min(10);
                replies.push(Ok(read_response(
                    address,
                    &profile[address..address + length],
                )));
                address += length;
            }
        }
        let mut fixture = Fixture(replies.into());

        let state = read_with(&mut fixture, Connection::Receiver);

        assert!(state.present && state.fresh && !state.wired);
        assert_eq!(state.percent, Some(72));
        assert_eq!(state.charging, Some(false));
        assert_eq!(state.dpi, Some(1600));
        assert_eq!(state.rate, Some(1000));
        assert!(state.identity.contains("firmware v1.2a"));
        assert!(fixture.0.is_empty());
    }

    #[test]
    fn battery_timeout_never_guesses_state_or_attempts_firmware() {
        let mut fixture = Fixture([Err("fixture timeout".into())].into());

        let state = read_with(&mut fixture, Connection::Wired);

        assert!(state.present && !state.fresh && state.wired);
        assert_eq!(state.percent, None);
        assert_eq!(state.charging, None);
        assert_eq!(state.error, "fixture timeout");
        assert!(fixture.0.is_empty());
    }

    #[test]
    fn new_model_battery_echo_does_not_become_zero_percent() {
        let mut fixture = Fixture([Ok(simple_request(BATTERY))].into());

        let state = read_with_model(&mut fixture, Connection::Receiver, F2_PRO_MAX);

        assert!(state.present && !state.fresh);
        assert_eq!(state.percent, None);
        assert_eq!(state.charging, None);
        assert!(state.error.contains("原样回显"));
        assert!(fixture.0.is_empty());
    }
}
