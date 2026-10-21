//! Read-only ATK X1 Pro Max protocol from OpenMouse `mouse-protocol` commit
//! `bac031567b28f2a55c616fc525b03f2d5fa452cc`.
//! Protocol facts and license attribution are recorded in docs/ATK-X1-PRO-MAX.md.
use peripheral_desk::{
    hid::{self, Interface},
    model::{Action, State},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "atk-x1-pro-max";
pub const REPORT_ID: u8 = 0x08;
pub const PAYLOAD_LEN: usize = 16;
pub const VID: u16 = 0x373b;
pub const WIRED_PID: u16 = 0x1017;
pub const RECEIVER_PID: u16 = 0x101b;
const USAGE_PAGE: u16 = 0xff02;
const USAGE: u16 = 0x0002;
const IDENTITY_COMMAND: u8 = 0x10;
const BATTERY_COMMAND: u8 = 0x04;
const CID: u8 = 0x02;
const MID: u8 = 0x27;
const IDENTITY_DATA_LEN: u8 = 3;
const BATTERY_DATA_LEN: u8 = 2;
const EEPROM_READ: u8 = 0x08;
const EEPROM_WRITE: u8 = 0x07;
const SYSTEM_ADDR: u16 = 0x0000;
const DPI_ADDR: u16 = 0x000c;
const REPORT_LEN: usize = PAYLOAD_LEN + 1;
const TIMEOUT: Duration = Duration::from_millis(700);

pub type Result<T> = std::result::Result<T, String>;

pub trait Transport {
    fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>>;
}

fn checksum(payload: &[u8; PAYLOAD_LEN]) -> u8 {
    let sum = payload[..15]
        .iter()
        .fold(REPORT_ID as u16, |sum, byte| sum + u16::from(*byte));
    (0x55u16.wrapping_sub(sum & 0xff)) as u8
}

fn checksum_valid(payload: &[u8]) -> bool {
    payload.len() == PAYLOAD_LEN
        && (payload
            .iter()
            .fold(REPORT_ID as u16, |sum, byte| sum + u16::from(*byte))
            & 0xff)
            == 0x55
}

fn command(command: u8) -> [u8; PAYLOAD_LEN] {
    let mut payload = [0; PAYLOAD_LEN];
    payload[0] = command;
    payload[15] = checksum(&payload);
    payload
}

fn parse_reply(response: &[u8], expected_command: u8, expected_data_len: u8) -> Result<Vec<u8>> {
    if !checksum_valid(response) {
        return Err("ATK 回复长度或 report-08 校验和不匹配".into());
    }
    if response[0] != expected_command {
        return Err(format!("ATK 回复命令不匹配：收到 0x{:02X}", response[0]));
    }
    if response[1] != 0 {
        return Err(format!(
            "ATK 命令 0x{expected_command:02X} 状态错误：0x{:02X}",
            response[1]
        ));
    }
    if response[4] != expected_data_len {
        return Err(format!(
            "ATK 命令 0x{expected_command:02X} 数据长度不匹配：期望 {expected_data_len}，收到 {}",
            response[4]
        ));
    }
    Ok(response[5..5 + expected_data_len as usize].to_vec())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Wired,
    Receiver,
}

impl Connection {
    pub fn from_pid(pid: u16) -> Option<Self> {
        match pid {
            WIRED_PID => Some(Self::Wired),
            RECEIVER_PID => Some(Self::Receiver),
            _ => None,
        }
    }

    fn wired(self) -> bool {
        self == Self::Wired
    }
    fn pid(self) -> u16 {
        if self.wired() {
            WIRED_PID
        } else {
            RECEIVER_PID
        }
    }
}

pub fn read_with<T: Transport>(transport: &mut T, connection: Connection) -> State {
    let mut state = State {
        key: DEVICE_KEY.into(),
        present: true,
        wired: connection.wired(),
        identity: format!(
            "ATK X1 Pro Max · USB {VID:04X}:{:04X} · CID/MID pending",
            connection.pid()
        ),
        settings_error: "只读电量与充电状态；此预览未开放设置写入。".into(),
        updated: now(),
        ..Default::default()
    };

    let identity = match transport.exchange(&command(IDENTITY_COMMAND)) {
        Ok(response) => match parse_reply(&response, IDENTITY_COMMAND, IDENTITY_DATA_LEN) {
            Ok(identity) => identity,
            Err(error) => {
                state.error = error;
                return state;
            }
        },
        Err(error) => {
            state.error = error;
            return state;
        }
    };
    if identity[0..2] != [CID, MID] {
        state.error = format!(
            "ATK USB 设备 CID/MID 不匹配：{},{}；未查询电量",
            identity[0], identity[1]
        );
        state.identity = format!("ATK 未支持型号 · USB {VID:04X}:{:04X}", connection.pid());
        return state;
    }
    state.identity = format!(
        "ATK X1 Pro Max · USB {VID:04X}:{:04X} · CID/MID 2,39",
        connection.pid()
    );

    let battery_data = match transport.exchange(&command(BATTERY_COMMAND)) {
        Ok(response) => match parse_reply(&response, BATTERY_COMMAND, BATTERY_DATA_LEN) {
            Ok(data) => data,
            Err(error) => {
                state.error = error;
                return state;
            }
        },
        Err(error) => {
            state.error = error;
            return state;
        }
    };
    if battery_data[0] > 100 {
        state.error = format!("ATK 电量超出 0–100：{}", battery_data[0]);
        return state;
    }
    let charging = match battery_data[1] {
        0 => false,
        1 => true,
        value => {
            state.error = format!("ATK 返回未知充电状态 0x{value:02X}");
            return state;
        }
    };
    state.percent = Some(battery_data[0]);
    state.charging = Some(charging);
    state.fresh = true;
    state
}

fn matching_interface(interface: &Interface) -> bool {
    interface.vid == VID
        && Connection::from_pid(interface.pid).is_some()
        && interface.usage_page == USAGE_PAGE
        && interface.usage == USAGE
        && interface.input_len as usize == REPORT_LEN
        && interface.output_len as usize == REPORT_LEN
        && interface.path.to_ascii_lowercase().contains("mi_01")
}

struct AtkTransport(peripheral_hid_transport::WinTransport);

impl Transport for AtkTransport {
    fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>> {
        self.0.flush()?;
        let mut wire = [0; REPORT_LEN];
        wire[0] = REPORT_ID;
        wire[1..].copy_from_slice(request);
        self.0.write(&wire, Duration::from_millis(200))?;
        let deadline = Instant::now() + TIMEOUT;
        while Instant::now() < deadline {
            let reply = self.0.read(
                REPORT_LEN,
                deadline.saturating_duration_since(Instant::now()),
            )?;
            if reply.is_empty() {
                break;
            }
            if reply.len() != REPORT_LEN {
                return Err(format!(
                    "ATK 输入报告长度不匹配：期望 {REPORT_LEN}，收到 {}",
                    reply.len()
                ));
            }
            if reply[0] != REPORT_ID || reply[1] != request[0] {
                continue;
            }
            return Ok(reply[1..].to_vec());
        }
        Err(format!(
            "ATK 未在 {} ms 内回复 report 08 命令 0x{:02X}",
            TIMEOUT.as_millis(),
            request[0]
        ))
    }
}

pub fn read_atk() -> Result<State> {
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces
        .iter()
        .filter(|interface| matching_interface(interface))
        .collect();
    match candidates.as_slice() {
        [] => Ok(State { key: DEVICE_KEY.into(), present: false, error: "未找到 ATK X1 Pro Max 精确 HID 集合（373B:1017/101B、FF02:0002、17 字节 input/output）".into(), settings_error: "只读电量与充电状态；此预览未开放设置写入。".into(), ..Default::default() }),
        [interface] => {
            let connection = Connection::from_pid(interface.pid).expect("candidate PID was checked");
            let mut transport = AtkTransport(peripheral_hid_transport::WinTransport::open(&interface.path)?);
            let mut state = read_with(&mut transport, connection);
            if state.fresh { populate_settings(&mut transport, &mut state)?; }
            Ok(state)
        }
        _ => Err("找到多个 ATK X1 Pro Max 协议集合；为避免查询错误接口，已停止".into()),
    }
}

fn read_eeprom<T: Transport>(transport: &mut T, address: u16, len: u8) -> Result<Vec<u8>> {
    let request = command(EEPROM_READ);
    let mut request = request;
    request[1] = 0;
    request[2] = (address >> 8) as u8;
    request[3] = address as u8;
    request[4] = len;
    request[15] = checksum(&request);
    let reply = transport.exchange(&request)?;
    let data = parse_reply(&reply, EEPROM_READ, len)?;
    if reply[2] != (address >> 8) as u8 || reply[3] != address as u8 {
        return Err("ATK EEPROM 地址读回不匹配".into());
    }
    Ok(data)
}

fn write_eeprom<T: Transport>(transport: &mut T, address: u16, data: &[u8]) -> Result<()> {
    if data.is_empty() || data.len() > 10 {
        return Err("ATK EEPROM 写入长度无效".into());
    }
    let mut request = command(EEPROM_WRITE);
    request[1] = 0;
    request[2] = (address >> 8) as u8;
    request[3] = address as u8;
    request[4] = data.len() as u8;
    request[5..5 + data.len()].copy_from_slice(data);
    request[15] = checksum(&request);
    let reply = transport.exchange(&request)?;
    parse_reply(&reply, EEPROM_WRITE, data.len() as u8)?;
    if reply[2] != (address >> 8) as u8 || reply[3] != address as u8 {
        return Err("ATK EEPROM 写入 ACK 地址不匹配".into());
    }
    Ok(())
}

fn pack_dpi(dpi: u16) -> [u8; 4] {
    let doubled = dpi > 30_000;
    let count = dpi / (if doubled { 100 } else { 50 }) - 1;
    let lo = count as u8;
    let nibble = (((count >> 8) as u8 & 3) << 2) | u8::from(doubled);
    let mode = nibble | (nibble << 4);
    let sum = lo.wrapping_mul(2).wrapping_add(mode);
    [lo, lo, mode, 0x55u8.wrapping_sub(sum)]
}

fn unpack_dpi(bytes: &[u8]) -> Result<u16> {
    if bytes.len() < 4
        || bytes[0]
            .wrapping_add(bytes[1])
            .wrapping_add(bytes[2])
            .wrapping_add(bytes[3])
            != 0x55
    {
        return Err("ATK DPI 校验失败".into());
    }
    let count = u16::from(bytes[0]) | (u16::from((bytes[2] & 0x0c) >> 2) << 8);
    let dpi = (count + 1) * 50;
    Ok(if bytes[2] & 1 != 0 { dpi * 2 } else { dpi })
}

fn valid_paw3950_dpi(dpi: u16) -> bool {
    (50..=30_000).contains(&dpi) && dpi.is_multiple_of(50)
        || (30_100..=36_000).contains(&dpi) && dpi.is_multiple_of(100)
}

fn populate_settings<T: Transport>(transport: &mut T, state: &mut State) -> Result<()> {
    let system = read_eeprom(transport, SYSTEM_ADDR, 6)?;
    let rates = [
        (0x08, 125),
        (0x04, 250),
        (0x02, 500),
        (0x01, 1000),
        (0x10, 2000),
        (0x20, 4000),
        (0x40, 8000),
    ];
    if system[0].wrapping_add(system[1]) == 0x55 {
        state.rate = rates
            .iter()
            .find(|(code, hz)| *code == system[0] && (!state.wired || *hz <= 1000))
            .map(|(_, hz)| *hz);
    }
    if system[2].wrapping_add(system[3]) != 0x55
        || system[4].wrapping_add(system[5]) != 0x55
        || system[2] == 0
        || system[2] > 6
        || system[4] >= system[2]
    {
        return Err("ATK 活动 DPI 档位或档位数校验失败".into());
    }
    let active = usize::from(system[4]);
    let stage = read_eeprom(transport, DPI_ADDR + (active as u16 * 4), 4)?;
    let dpi = unpack_dpi(&stage)?;
    if valid_paw3950_dpi(dpi) {
        state.dpi = Some(dpi);
    }
    if state.rate.is_none() || state.dpi.is_none() {
        state.settings_error =
            "ATK EEPROM returned an unsupported or invalid setting; no value is shown".into();
    }
    Ok(())
}

fn apply_transaction<T: Transport>(
    transport: &mut T,
    address: u16,
    before: &[u8],
    after: &[u8],
) -> Result<()> {
    let attempt = write_eeprom(transport, address, after).and_then(|_| {
        let readback = read_eeprom(transport, address, before.len() as u8)?;
        if readback == after {
            Ok(())
        } else {
            Err("ATK EEPROM readback differed".into())
        }
    });
    if let Err(error) = attempt {
        let rollback = write_eeprom(transport, address, before)
            .and_then(|_| read_eeprom(transport, address, before.len() as u8))
            .and_then(|actual| {
                if actual == before {
                    Ok(())
                } else {
                    Err("rollback readback differed".into())
                }
            });
        return match rollback {
            Ok(()) => Err(format!(
                "ATK setting failed: {error}; original bytes restored and verified"
            )),
            Err(rollback) => Err(format!(
                "ATK setting failed: {error}; rollback not verified: {rollback}"
            )),
        };
    }
    Ok(())
}

pub fn apply_action(action: Action) -> Result<()> {
    let interfaces = hid::enumerate()?;
    let mut candidates = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = candidates.next() else {
        return Err("未找到精确 ATK HID 集合".into());
    };
    if candidates.next().is_some() {
        return Err("找到多个 ATK 协议集合，已停止写入".into());
    }
    Connection::from_pid(interface.pid).ok_or("ATK PID 不受支持")?;
    let mut transport = AtkTransport(peripheral_hid_transport::WinTransport::open(
        &interface.path,
    )?);
    let identity = parse_reply(
        &transport.exchange(&command(IDENTITY_COMMAND))?,
        IDENTITY_COMMAND,
        IDENTITY_DATA_LEN,
    )?;
    let (expected_cid, expected_mid) = (0x02, 0x27);
    if identity.get(0..2) != Some(&[expected_cid, expected_mid]) {
        return Err("ATK CID/MID mismatch; refusing settings write".into());
    }
    match action {
        Action::Rate(rate) => {
            if rate > if expected_mid == 0x53 { 1000 } else { 8000 } {
                return Err("ATK polling rate exceeds this model/receiver limit".into());
            }
            let code = match rate {
                125 => 0x08,
                250 => 0x04,
                500 => 0x02,
                1000 => 0x01,
                2000 => 0x10,
                4000 => 0x20,
                8000 => 0x40,
                _ => {
                    return Err(
                        "ATK polling rate must be 125/250/500/1000/2000/4000/8000 Hz".into(),
                    )
                }
            };
            let before = read_eeprom(&mut transport, SYSTEM_ADDR, 2)?;
            let after = [code, 0x55u8.wrapping_sub(code)].to_vec();
            apply_transaction(&mut transport, SYSTEM_ADDR, &before, &after)
        }
        Action::Dpi(dpi) => {
            if !valid_paw3950_dpi(dpi) {
                return Err("ATK X1 Pro Max DPI must be 50–30000 in 50-DPI steps or 30100–36000 in 100-DPI steps".into());
            }
            let system = read_eeprom(&mut transport, SYSTEM_ADDR, 6)?;
            if system[2].wrapping_add(system[3]) != 0x55
                || system[4].wrapping_add(system[5]) != 0x55
                || system[2] == 0
                || system[2] > 6
                || system[4] >= system[2]
            {
                return Err("ATK 活动 DPI 档位或档位数校验失败；拒绝写入".into());
            }
            let address = DPI_ADDR + u16::from(system[4]) * 4;
            let before = read_eeprom(&mut transport, address, 4)?;
            let after = pack_dpi(dpi).to_vec();
            apply_transaction(&mut transport, address, &before, &after)
        }
        _ => Err("This ATK preview supports DPI and polling-rate settings only".into()),
    }
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
    use std::collections::VecDeque;

    fn response(command: u8, data: &[u8]) -> [u8; PAYLOAD_LEN] {
        let mut frame = [0; PAYLOAD_LEN];
        frame[0] = command;
        frame[4] = data.len() as u8;
        frame[5..5 + data.len()].copy_from_slice(data);
        frame[15] = checksum(&frame);
        frame
    }

    struct Fixture(VecDeque<[u8; PAYLOAD_LEN]>, Vec<u8>);
    impl Transport for Fixture {
        fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>> {
            assert!(checksum_valid(request));
            self.1.push(request[0]);
            Ok(self.0.pop_front().unwrap().to_vec())
        }
    }

    #[test]
    fn only_exact_verified_transports_are_declared() {
        assert_eq!(Connection::from_pid(WIRED_PID), Some(Connection::Wired));
        assert_eq!(
            Connection::from_pid(RECEIVER_PID),
            Some(Connection::Receiver)
        );
        for pid in [0x101a, 0x1016, 0x1018, 0x102f, 0x103d] {
            assert_eq!(Connection::from_pid(pid), None);
        }
    }

    #[test]
    fn only_the_exact_interface_one_report_shape_is_selected() {
        let interface = Interface {
            path: "\\\\?\\hid#vid_373b&pid_1017&mi_01&col01".into(),
            vid: VID,
            pid: WIRED_PID,
            usage_page: USAGE_PAGE,
            usage: USAGE,
            feature_len: 0,
            input_len: REPORT_LEN as u16,
            output_len: REPORT_LEN as u16,
            product: "ATK X1 Pro Max".into(),
        };
        assert!(matching_interface(&interface));
        let mut other = interface.clone();
        other.pid = 0x101a;
        assert!(!matching_interface(&other));
        other = interface.clone();
        other.usage_page = 0xff04;
        assert!(!matching_interface(&other));
        other = interface.clone();
        other.path = other.path.replace("mi_01", "mi_00");
        assert!(!matching_interface(&other));
        other = interface.clone();
        other.input_len = PAYLOAD_LEN as u16;
        assert!(!matching_interface(&other));
    }

    #[test]
    fn exact_identity_is_required_before_battery_is_queried() {
        let mut fixture = Fixture(
            [
                response(IDENTITY_COMMAND, &[2, 0x26, 0x2c]),
                response(BATTERY_COMMAND, &[80, 1]),
            ]
            .into(),
            vec![],
        );
        let state = read_with(&mut fixture, Connection::Receiver);
        assert!(!state.fresh);
        assert_eq!(fixture.1, [IDENTITY_COMMAND]);
        assert_eq!(state.percent, None);
    }

    #[test]
    fn strict_identity_and_battery_reply_yield_read_only_state() {
        // Captured report-08 payloads from wired.json, with report ID 0x08 stripped.
        let mut fixture = Fixture(
            [
                [
                    0x10, 0x00, 0x00, 0x00, 0x03, 0x02, 0x27, 0x2c, 0, 0, 0, 0, 0, 0, 0, 0xe5,
                ],
                [
                    0x04, 0x00, 0x00, 0x00, 0x02, 0x2d, 0x01, 0x0f, 0x06, 0, 0, 0, 0, 0, 0, 0x04,
                ],
            ]
            .into(),
            vec![],
        );
        let state = read_with(&mut fixture, Connection::Wired);
        assert!(state.present && state.fresh && state.wired);
        assert_eq!(state.percent, Some(45));
        assert_eq!(state.charging, Some(true));
        assert_eq!(fixture.1, [IDENTITY_COMMAND, BATTERY_COMMAND]);
    }

    #[test]
    fn captured_receiver_replies_report_discharging_and_ignore_padding() {
        // Captured receiver.json report-08 payloads with the report-ID byte stripped.
        let mut fixture = Fixture(
            [
                [
                    0x10, 0, 0, 0, 3, 0x02, 0x27, 0x2c, 0, 0, 0, 0, 0, 0, 0, 0xe5,
                ],
                [
                    0x04, 0, 0, 0, 2, 0x19, 0, 0x0e, 0x46, 0, 0, 0, 0, 0, 0, 0xda,
                ],
            ]
            .into(),
            vec![],
        );
        let state = read_with(&mut fixture, Connection::Receiver);
        assert!(state.present && state.fresh && !state.wired);
        assert_eq!(state.percent, Some(25));
        assert_eq!(state.charging, Some(false));
        assert_eq!(fixture.1, [IDENTITY_COMMAND, BATTERY_COMMAND]);
    }

    #[test]
    fn malformed_status_length_checksum_and_battery_values_are_rejected() {
        let mut wrong_status = response(IDENTITY_COMMAND, &[2, 0x27, 0x2c]);
        wrong_status[1] = 1;
        wrong_status[15] = checksum(&wrong_status);
        assert!(parse_reply(&wrong_status, IDENTITY_COMMAND, IDENTITY_DATA_LEN).is_err());
        assert!(parse_reply(
            &response(IDENTITY_COMMAND, &[2, 0x27]),
            IDENTITY_COMMAND,
            IDENTITY_DATA_LEN
        )
        .is_err());
        let mut bad_checksum = response(BATTERY_COMMAND, &[45, 1]);
        bad_checksum[15] ^= 1;
        assert!(parse_reply(&bad_checksum, BATTERY_COMMAND, 2).is_err());
        assert!(parse_reply(&response(BATTERY_COMMAND, &[45, 1, 9]), BATTERY_COMMAND, 2).is_err());
        let mut invalid_percent = Fixture(
            [
                response(IDENTITY_COMMAND, &[2, 0x27, 0x2c]),
                response(BATTERY_COMMAND, &[101, 1]),
            ]
            .into(),
            vec![],
        );
        let state = read_with(&mut invalid_percent, Connection::Wired);
        assert!(!state.fresh);
        assert_eq!(state.percent, None);
        let mut invalid_charge = Fixture(
            [
                response(IDENTITY_COMMAND, &[2, 0x27, 0x2c]),
                response(BATTERY_COMMAND, &[50, 2]),
            ]
            .into(),
            vec![],
        );
        let state = read_with(&mut invalid_charge, Connection::Wired);
        assert!(!state.fresh);
        assert_eq!(state.percent, None);
    }

    #[test]
    fn paw3950_dpi_values_round_trip_with_the_documented_50_dpi_grid() {
        for dpi in [50, 400, 1_200, 12_800, 30_000, 36_000] {
            assert_eq!(unpack_dpi(&pack_dpi(dpi)).unwrap(), dpi);
        }
    }
}
