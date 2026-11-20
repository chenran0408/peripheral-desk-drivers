//! First status block of the MCHOSE CZ receiver. The caller selects the model and HID interface.
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant};

pub const LEN: usize = 65;

#[derive(Debug, PartialEq)]
pub struct Battery {
    pub raw: u8,
    pub percent: u8,
    pub charging: bool,
}

fn first_block_query() -> [u8; LEN] {
    let mut request = [0; LEN];
    request[..6].copy_from_slice(&[0, 0x55, 0x05, 0, 0x38, 0x38]);
    request
}

fn parse(reply: &[u8]) -> Option<Battery> {
    if reply.len() != LEN
        || reply[..4] != [0, 0xaa, 0x05, 0]
        || reply[5..9] != [0x38, 0, 0, 0]
        || reply[11..13] != [0xaa, 0xbb]
    {
        return None;
    }
    let raw = reply[41];
    let percent = raw & 0x7f;
    (1..=100).contains(&percent).then_some(Battery {
        raw,
        percent,
        charging: raw & 0x80 != 0,
    })
}

pub fn read_battery(path: &str) -> Result<Battery, String> {
    let mut transport = WinTransport::open(path)?;
    transport.flush()?;
    transport.write(&first_block_query(), Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(1200);
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        if let Some(battery) = parse(&transport.read(LEN, remaining)?) {
            return Ok(battery);
        }
    }
    Err("接收器未返回有效的 CZ 电量状态块".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_the_requested_status_block() {
        let mut reply = first_block_query();
        assert_eq!(parse(&reply), None); // output echo
        reply[..6].copy_from_slice(&[0, 0xaa, 5, 0, 0x92, 0x38]);
        reply[11..13].copy_from_slice(&[0xaa, 0xbb]);
        for (raw, percent, charging) in [
            (0x59, 89, false),
            (0xce, 78, true),
            (0x64, 100, false),
            (0xe4, 100, true),
        ] {
            reply[41] = raw;
            assert_eq!(
                parse(&reply),
                Some(Battery {
                    raw,
                    percent,
                    charging
                })
            );
        }
        reply[6] = 0x38;
        assert_eq!(parse(&reply), None);
        reply[6] = 0;
        reply[12] = 0;
        assert_eq!(parse(&reply), None);
        reply[12] = 0xbb;
        for raw in [0, 0x80, 0xff] {
            reply[41] = raw;
            assert_eq!(parse(&reply), None);
        }
    }
}
