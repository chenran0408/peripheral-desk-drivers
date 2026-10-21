//! Read-only ATK A9 Plus Nearlink telemetry based on the captured report-08
//! frames documented in docs/ATK-A9-PLUS-NEARLINK.md.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "atk-a9-plus-nearlink";
pub const REPORT_ID: u8 = 0x08;
pub const PAYLOAD_LEN: usize = 16;
pub const VID: u16 = 0x373b;
pub const WIRED_PID: u16 = 0x1115;
pub const RECEIVER_PID: u16 = 0x10c9;
const USAGE_PAGE: u16 = 0xff02;
const USAGE: u16 = 0x0002;
const IDENTITY_COMMAND: u8 = 0x10;
const BATTERY_COMMAND: u8 = 0x04;
const CID: u8 = 0x02;
const MID: u8 = 0x53;
const IDENTITY_DATA_LEN: u8 = 2;
const BATTERY_DATA_LEN: u8 = 2;
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
            "ATK A9 Plus Nearlink · USB {VID:04X}:{:04X} · CID/MID pending",
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
        "ATK A9 Plus Nearlink · USB {VID:04X}:{:04X} · CID/MID 2,83",
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
        [] => Ok(State {
            key: DEVICE_KEY.into(),
            present: false,
            error: "未找到 ATK A9 Plus Nearlink 精确 HID 集合（373B:1115/10C9、FF02:0002、17 字节 input/output）".into(),
            settings_error: "只读电量与充电状态；此预览未开放设置写入。".into(),
            ..Default::default()
        }),
        [interface] => {
            let connection = Connection::from_pid(interface.pid).expect("candidate PID was checked");
            let mut transport = AtkTransport(peripheral_hid_transport::WinTransport::open(&interface.path)?);
            Ok(read_with(&mut transport, connection))
        }
        _ => Err("找到多个 ATK A9 Plus Nearlink 协议集合；为避免查询错误接口，已停止".into()),
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

    struct Fixture(VecDeque<[u8; PAYLOAD_LEN]>, Vec<u8>);

    impl Transport for Fixture {
        fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>> {
            assert!(checksum_valid(request));
            self.1.push(request[0]);
            Ok(self.0.pop_front().expect("fixture reply").to_vec())
        }
    }

    fn frames(battery: [u8; PAYLOAD_LEN]) -> Fixture {
        Fixture(
            [
                [
                    0x10, 0x00, 0x00, 0x00, 0x02, 0x02, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0xe6,
                ],
                battery,
            ]
            .into(),
            vec![],
        )
    }

    #[test]
    fn all_four_captured_firmware_and_connection_paths_decode_declared_battery_bytes() {
        // Raw report-08 replies from the four pinned captures, with report ID stripped.
        let captures = [
            (
                frames([
                    0x04, 0, 0, 0, 2, 0x64, 1, 0x10, 0x8e, 0, 0, 0, 0, 0, 0, 0x44,
                ]),
                Connection::Wired,
                true,
            ),
            (
                frames([
                    0x04, 0, 0, 0, 2, 0x64, 0, 0x10, 0x8e, 0, 0, 0, 0, 0, 0, 0x45,
                ]),
                Connection::Receiver,
                false,
            ),
            (
                frames([
                    0x04, 0, 0, 0, 2, 0x64, 1, 0x10, 0x9d, 0, 0, 0, 0, 0, 0, 0x35,
                ]),
                Connection::Wired,
                true,
            ),
            (
                frames([
                    0x04, 0, 0, 0, 2, 0x64, 0, 0x10, 0x9a, 0, 0, 0, 0, 0, 0, 0x39,
                ]),
                Connection::Receiver,
                false,
            ),
        ];

        for (mut fixture, connection, charging) in captures {
            let state = read_with(&mut fixture, connection);
            assert!(state.present && state.fresh);
            assert_eq!(state.wired, connection == Connection::Wired);
            assert_eq!(state.percent, Some(100));
            assert_eq!(state.charging, Some(charging));
            assert_eq!(fixture.1, [IDENTITY_COMMAND, BATTERY_COMMAND]);
        }
    }

    #[test]
    fn identity_requires_declared_two_byte_payload_and_exact_cid_mid() {
        let mut short_identity = frames([0; PAYLOAD_LEN]);
        short_identity.0[0][4] = 1;
        short_identity.0[0][15] = checksum(&short_identity.0[0]);
        let state = read_with(&mut short_identity, Connection::Wired);
        assert!(!state.fresh);
        assert_eq!(short_identity.1, [IDENTITY_COMMAND]);

        let mut unknown_identity = frames([0; PAYLOAD_LEN]);
        unknown_identity.0[0][6] = 0x27;
        unknown_identity.0[0][15] = checksum(&unknown_identity.0[0]);
        let state = read_with(&mut unknown_identity, Connection::Receiver);
        assert!(!state.fresh);
        assert_eq!(unknown_identity.1, [IDENTITY_COMMAND]);
    }

    #[test]
    fn only_verified_product_ids_are_supported() {
        assert_eq!(Connection::from_pid(WIRED_PID), Some(Connection::Wired));
        assert_eq!(
            Connection::from_pid(RECEIVER_PID),
            Some(Connection::Receiver)
        );
        for pid in [0x101a, 0x101b, 0x1224, 0x1263, 0x1300] {
            assert_eq!(Connection::from_pid(pid), None);
        }
    }
}
