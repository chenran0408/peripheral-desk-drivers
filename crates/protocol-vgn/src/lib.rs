//! Read-only VGN Dragonfly F2 Master+ protocol facts from OpenMouse
//! `mouse-protocol` commit `bac031567b28f2a55c616fc525b03f2d5fa452cc`.
//! See `docs/VGN-F2-MASTER-PLUS.md` for source and license attribution.
use peripheral_desk::model::State;
use std::time::{SystemTime, UNIX_EPOCH};

pub type Result<T> = std::result::Result<T, String>;

pub const DEVICE_KEY: &str = "vgn-f2-master-plus";
pub const REPORT_ID: u8 = 0x08;
pub const PAYLOAD_LEN: usize = 16;
pub const VID: u16 = 0x3554;
pub const RECEIVER_PID: u16 = 0xfb56;
pub const WIRED_PID: u16 = 0xfb57;
const BATTERY: u8 = 0x04;
const FIRMWARE: u8 = 0x12;

/// A serialized report-08 transaction. The Windows transport adds/removes the
/// report-ID byte and must cancel and drain timed-out overlapped I/O.
pub trait Transport {
    fn exchange(&mut self, request: &[u8; PAYLOAD_LEN]) -> Result<Vec<u8>>;
}

mod transport;
pub use transport::read_vgn;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Receiver,
    Wired,
}

impl Connection {
    pub fn from_pid(pid: u16) -> Option<Self> {
        match pid {
            RECEIVER_PID => Some(Self::Receiver),
            WIRED_PID => Some(Self::Wired),
            _ => None,
        }
    }

    fn wired(self) -> bool {
        self == Self::Wired
    }

    fn pid(self) -> u16 {
        match self {
            Self::Receiver => RECEIVER_PID,
            Self::Wired => WIRED_PID,
        }
    }
}

pub fn read_with<T: Transport>(transport: &mut T, connection: Connection) -> State {
    let mut state = State {
        key: DEVICE_KEY.into(),
        present: true,
        wired: connection.wired(),
        identity: format!(
            "VGN Dragonfly F2 Master+ · USB {:04X}:{:04X} · report 08 · firmware unknown",
            VID,
            connection.pid()
        ),
        settings_error: "仅提供只读电量、充电状态和公开固件版本；未开放设置写入。".into(),
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
            assert!(request[0] == BATTERY || request[0] == FIRMWARE);
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
        let mut fixture = Fixture([Ok(battery), Ok(firmware)].into());

        let state = read_with(&mut fixture, Connection::Receiver);

        assert!(state.present && state.fresh && !state.wired);
        assert_eq!(state.percent, Some(72));
        assert_eq!(state.charging, Some(false));
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
}
