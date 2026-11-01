//! Barracuda Pro PA exchange decoded from HaloBattery's hardware capture (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub const KEY: &str = "razer-barracuda-pro";
const LEN: usize = 64;
pub fn query(command: u8) -> [u8; LEN] {
    let mut r = [0; LEN];
    r[..9].copy_from_slice(&[1, 0x80, 8, b'P', b'A', 8, 8, 3, command]);
    r
}
pub fn remote(on: bool) -> [u8; LEN] {
    let mut r = [0; LEN];
    r[..10].copy_from_slice(&[1, 0x80, 7, b'P', b'A', 0x0e, 8, 2, 0xe1, u8::from(on)]);
    r
}
pub fn parse(reply: &[u8], command: u8) -> Option<u8> {
    let r = reply.strip_prefix(&[1]).unwrap_or(reply);
    // Replies include their report ID on Windows, but some HID stacks strip it.
    let offset = if r.len() == reply.len() { 0 } else { 1 };
    let marker = 3usize.checked_sub(offset)?;
    let echo = 13usize.checked_sub(offset)?;
    if r.len() <= echo + 3
        || r.get(marker..marker + 2)? != b"PI"
        || r[echo] != command
        || !matches!(r[echo + 1], 1 | 2)
        || r[echo + 2] == 0
        || r[echo + 2] > 32
    {
        return None;
    }
    let value = r[echo + 3];
    if command == 0x21 && value > 100 {
        return None;
    }
    Some(value)
}
fn query_one(t: &mut WinTransport, command: u8) -> Result<u8, String> {
    t.flush()?;
    t.write(&query(command), Duration::from_millis(500))?;
    let end = Instant::now() + Duration::from_millis(900);
    while Instant::now() < end {
        if let Some(value) = parse(&t.read(LEN, Duration::from_millis(100))?, command) {
            return Ok(value);
        }
    }
    Err("Barracuda Pro 无有效回复（耳机可能离线）".into())
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let choices: Vec<_> = interfaces
        .iter()
        .filter(|i| {
            i.vid == 0x1532
                && i.pid == 0x053a
                && i.usage_page == 0xff00
                && i.input_len as usize >= LEN
                && i.output_len as usize >= LEN
        })
        .collect();
    if choices.is_empty() {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 Barracuda Pro 2.4 GHz 接收器".into(),
            ..State::default()
        });
    }
    for i in choices {
        let mut t = WinTransport::open(&i.path)?;
        t.write(&remote(true), Duration::from_millis(500))?;
        std::thread::sleep(Duration::from_millis(50));
        let reading: Result<(u8, bool), String> =
            (|| Ok((query_one(&mut t, 0x21)?, query_one(&mut t, 0x2a)? != 0)))();
        t.write(&remote(false), Duration::from_millis(500))?;
        if let Ok((percent, charging)) = reading {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                identity: "Razer Barracuda Pro · 1532:053A · 2.4 GHz".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅提供只读电量和充电状态；查询需要临时远程会话。".into(),
                ..State::default()
            });
        }
    }
    Err("Barracuda Pro 未回复有效电量".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pa_layout() {
        assert_eq!(&query(0x21)[..9], &[1, 0x80, 8, b'P', b'A', 8, 8, 3, 0x21]);
        let mut r = [0; 64];
        r[0] = 1;
        r[3] = b'P';
        r[4] = b'I';
        r[13] = 0x21;
        r[14] = 1;
        r[15] = 1;
        r[16] = 27;
        assert_eq!(parse(&r, 0x21), Some(27));
    }
}
