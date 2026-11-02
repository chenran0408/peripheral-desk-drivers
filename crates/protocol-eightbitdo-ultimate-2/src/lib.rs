use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const KEY: &str = "eightbitdo-ultimate-2-wireless";
const VENDOR_ID: u16 = 0x2dc8;
const PRODUCT_ID: u16 = 0x6012;
const BATTERY_REPORT_LEN: usize = 34;
const BATTERY_OFFSET: usize = 14;

pub fn parse_report(report: &[u8]) -> Option<(u8, bool)> {
    if report.len() < BATTERY_REPORT_LEN || !matches!(report[0], 0x01 | 0x04) {
        return None;
    }
    let battery = report[BATTERY_OFFSET];
    let percent = battery & 0x7f;
    (1..=100)
        .contains(&percent)
        .then_some((percent, battery & 0x80 != 0 && percent < 100))
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let interface = interfaces
        .iter()
        .find(|item| {
            item.vid == VENDOR_ID
                && item.pid == PRODUCT_ID
                && item.input_len as usize >= BATTERY_REPORT_LEN
                && item.usage_page == 0x01
                && matches!(item.usage, 0x04 | 0x05)
        })
        .ok_or("未找到 8BitDo Ultimate 2 输入接口")?;

    let mut transport = WinTransport::open(&interface.path)?;
    let end = Instant::now() + Duration::from_millis(500);
    while Instant::now() < end {
        let report = transport.read(interface.input_len as usize, Duration::from_millis(50))?;
        if let Some((percent, charging)) = parse_report(&report) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: false,
                identity: format!("8BitDo Ultimate 2 Wireless · 2DC8:{PRODUCT_ID:04X}"),
                settings_error: "被动读取输入报告；不会主动切换手柄模式。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            });
        }
    }

    Ok(State {
        key: KEY.into(),
        present: true,
        identity: format!("8BitDo Ultimate 2 Wireless · 2DC8:{PRODUCT_ID:04X}"),
        error: "未收到包含有效电量的 34 字节增强报告；可能是旧固件或手柄休眠。".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_enhanced_battery_report() {
        let mut report = [0u8; BATTERY_REPORT_LEN];
        report[0] = 0x04;
        report[BATTERY_OFFSET] = 0x80 | 62;
        assert_eq!(parse_report(&report), Some((62, true)));

        report[BATTERY_OFFSET] = 0x80 | 100;
        assert_eq!(parse_report(&report), Some((100, false)));
    }

    #[test]
    fn rejects_short_unknown_and_empty_reports() {
        let mut short = [0u8; 12];
        short[0] = 0x04;
        assert_eq!(parse_report(&short), None);

        let mut unknown = [0u8; BATTERY_REPORT_LEN];
        unknown[0] = 0x03;
        unknown[BATTERY_OFFSET] = 50;
        assert_eq!(parse_report(&unknown), None);

        let mut empty = [0u8; BATTERY_REPORT_LEN];
        empty[0] = 0x01;
        assert_eq!(parse_report(&empty), None);
    }
}
