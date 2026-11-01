//! SteelSeries mouse telemetry from HaloBattery, rivalcfg (WTFPL), and steel-mouse.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const AEROX3_KEY: &str = "steelseries-aerox-3-wireless";
pub const AEROX5_KEY: &str = "steelseries-aerox-5-wireless";
pub const RIVAL3_KEY: &str = "steelseries-rival-3-wireless";
const LEN: usize = 64;
pub fn request(aerox: bool) -> [u8; LEN] {
    let mut frame = [0; LEN];
    frame[1] = if aerox { 0xd2 } else { 0xaa };
    if !aerox {
        frame[2] = 1;
    }
    frame
}
pub fn parse(reply: &[u8], aerox: bool) -> Option<(u8, bool)> {
    let r = reply.strip_prefix(&[0]).unwrap_or(reply);
    if aerox {
        if r.len() < 2 || r[0] != 0xd2 {
            return None;
        }
        let value = r[1] & 0x7f;
        if value == 0 || value > 100 {
            return None;
        }
        let percent = if value <= 21 { (value - 1) * 5 } else { value };
        return Some((percent, r[1] & 0x80 != 0));
    }
    if r.len() >= 4 && r[0] == 0xaa && r[1] <= 100 {
        return Some((r[1], r[3] != 0));
    }
    if r.len() >= 3 && r[0] <= 100 && r[2] <= 1 {
        return Some((r[0], r[2] == 1));
    }
    None
}
fn candidate(i: &hid::Interface, key: &str) -> bool {
    let pid = match key {
        AEROX3_KEY => matches!(i.pid, 0x1838 | 0x1878),
        AEROX5_KEY => matches!(i.pid, 0x1852 | 0x185c | 0x1860),
        RIVAL3_KEY => matches!(i.pid, 0x1830 | 0x1872),
        _ => false,
    };
    i.vid == 0x1038
        && pid
        && i.usage_page == 0xffc0
        && i.usage == 1
        && i.input_len as usize >= LEN
        && i.output_len as usize >= LEN
}
pub fn read_state(key: &str) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let choices: Vec<_> = interfaces.iter().filter(|i| candidate(i, key)).collect();
    if choices.is_empty() {
        return Ok(State {
            key: key.into(),
            error: "未找到 SteelSeries 鼠标指定电量接口".into(),
            ..State::default()
        });
    }
    let aerox = key != RIVAL3_KEY;
    for i in choices {
        let mut transport = WinTransport::open(&i.path)?;
        transport.flush()?;
        transport.write(&request(aerox), Duration::from_millis(500))?;
        let end = Instant::now() + Duration::from_millis(900);
        while Instant::now() < end {
            if let Some((percent, charging)) = parse(
                &transport.read(i.input_len as usize, Duration::from_millis(100))?,
                aerox,
            ) {
                return Ok(State {
                    key: key.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    identity: format!("SteelSeries mouse · 1038:{:04X}", i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅提供只读电量与充电状态。".into(),
                    ..State::default()
                });
            }
        }
    }
    Err("SteelSeries 鼠标未回复有效电量（可能休眠）".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_exchanges() {
        assert_eq!(&request(true)[..3], &[0, 0xd2, 0]);
        assert_eq!(parse(&[0xd2, 0x84], true), Some((15, true)));
        assert_eq!(parse(&[0xd2, 75], true), Some((75, false)));
        assert_eq!(parse(&[0xd2, 0], true), None);
        assert_eq!(parse(&[0xaa, 56, 0, 1], false), Some((56, true)));
        assert_eq!(parse(&[56, 0, 0], false), Some((56, false)));
        assert_eq!(parse(&[56, 0, 2], false), None);
    }
}
