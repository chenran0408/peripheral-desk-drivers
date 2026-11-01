//! ATK A9 Air Master+ receiver battery adapter.
//! Protocol facts are independently implemented from Voltide v2.1.2 (MIT);
//! see UPSTREAM.md and THIRD_PARTY_LICENSES/Voltide-LICENSE.txt. Peripheral Desk hardware testing has
//! not been performed.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_SetOutputReport,
    Foundation::{CloseHandle, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};

const KEY: &str = "atk-a9-air-master-plus";
const VID: u16 = 0x373b;
const PID: u16 = 0x1278;
const USAGE_PAGE: u16 = 0xff02;
const USAGE: u16 = 0x0002;
const REPORT_ID: u8 = 0x08;
const COMMAND_BATTERY: u8 = 0x04;
const REPORT_LEN: usize = 17;
const CHECKSUM_TOTAL: u8 = 0x4d;

fn supported(interface: &hid::Interface) -> bool {
    interface.vid == VID
        && interface.pid == PID
        && interface.usage_page == USAGE_PAGE
        && interface.usage == USAGE
        && interface.input_len as usize == REPORT_LEN
        && interface.output_len as usize == REPORT_LEN
}

fn checksum(bytes: &[u8]) -> u8 {
    CHECKSUM_TOTAL.wrapping_sub(
        bytes[..REPORT_LEN - 2]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte)),
    )
}

fn query() -> [u8; REPORT_LEN] {
    let mut report = [0; REPORT_LEN];
    report[0] = REPORT_ID;
    report[1] = COMMAND_BATTERY;
    report[REPORT_LEN - 1] = checksum(&report[1..]);
    report
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Reading {
    percent: u8,
    charging: Option<bool>,
    millivolts: u16,
}

fn parse_reply(report: &[u8]) -> Option<Reading> {
    if report.len() != REPORT_LEN
        || report[0] != REPORT_ID
        || report[1] != COMMAND_BATTERY
        || report[1..]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte))
            != CHECKSUM_TOTAL
    {
        return None;
    }
    let percent = report[6];
    if percent > 100 {
        return None;
    }
    let charging = match report[7] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Some(Reading {
        percent,
        charging,
        millivolts: u16::from_be_bytes([report[8], report[9]]),
    })
}

fn send_query(path: &str, report: &[u8; REPORT_LEN]) -> Result<(), String> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    unsafe {
        let handle = CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            return Err(format!(
                "ATK A9 Air 查询通道打开失败：{}",
                std::io::Error::last_os_error()
            ));
        }
        let sent = HidD_SetOutputReport(handle, report.as_ptr().cast(), REPORT_LEN as u32);
        let error = std::io::Error::last_os_error();
        CloseHandle(handle);
        if !sent {
            Err(format!("ATK A9 Air 电量查询发送失败：{error}"))
        } else {
            Ok(())
        }
    }
}

fn read_reply(io: &mut WinTransport) -> Result<Option<Reading>, String> {
    let until = Instant::now() + Duration::from_millis(700);
    while Instant::now() < until {
        let remaining = until.saturating_duration_since(Instant::now());
        let report = io.read(REPORT_LEN, remaining.min(Duration::from_millis(100)))?;
        if let Some(reading) = parse_reply(&report) {
            return Ok(Some(reading));
        }
    }
    Ok(None)
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces.iter().filter(|item| supported(item)).collect();
    let Some(interface) = candidates.first().copied() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 ATK A9 Air 大师+ 2.4 GHz 接收器的 FF02:0002 电量通道".into(),
            settings_error:
                "该插件只读取接收器电量、充电标志和电压；不提供 DPI、回报率、改键、宏或灯效控制。"
                    .into(),
            ..State::default()
        });
    };
    if candidates.len() != 1 {
        return Err("发现多个 ATK A9 Air 大师+ 电量通道，停止查询".into());
    }

    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    let request = query();
    let mut reading = None;
    for _ in 0..3 {
        send_query(&interface.path, &request)?;
        if let Some(parsed) = read_reply(&mut io)? {
            reading = Some(parsed);
            break;
        }
        // The upstream adapter retries this read-only query up to three times.
        // Drain any late response before reissuing so it cannot be mistaken for
        // the next request's reply.
        io.flush()?;
    }
    let reading = reading.ok_or("ATK A9 Air 大师+ 未收到有效的校验通过电量回复")?;

    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(reading.percent),
        charging: reading.charging,
        wired: false,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: format!(
            "ATK A9 Air 大师+ · 2.4 GHz 接收器 · {} mV",
            reading.millivolts
        ),
        settings_error: "电量、充电标志和电压为只读；DPI、回报率、改键、宏和灯效未适配。".into(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err(
                "ATK A9 Air 大师+ 插件只读接收器电量、充电状态与电压，不执行其他设备设置".into(),
            );
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("atk-a9-air-master-plus", handler);

#[cfg(test)]
mod tests {
    use super::*;

    fn captured_reply(percent: u8, charge: u8, millivolts: u16) -> [u8; REPORT_LEN] {
        let mut report = [0; REPORT_LEN];
        report[0] = REPORT_ID;
        report[1] = COMMAND_BATTERY;
        report[5] = 2;
        report[6] = percent;
        report[7] = charge;
        report[8..10].copy_from_slice(&millivolts.to_be_bytes());
        report[REPORT_LEN - 1] = checksum(&report[1..]);
        report
    }

    #[test]
    fn query_and_captured_reply_match_voltide_v2_1_2() {
        let query = query();
        assert_eq!(&query[..2], &[REPORT_ID, COMMAND_BATTERY]);
        assert_eq!(query[16], 0x49);
        assert_eq!(
            query[1..]
                .iter()
                .fold(0u8, |sum, byte| sum.wrapping_add(*byte)),
            CHECKSUM_TOTAL
        );

        // Voltide v2.1.2's contributor capture, also shown in its source comment.
        let capture = [
            0x08, 0x04, 0x00, 0x00, 0x00, 0x02, 0x64, 0x00, 0x10, 0xDD, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xF6,
        ];
        assert_eq!(
            parse_reply(&capture),
            Some(Reading {
                percent: 100,
                charging: Some(false),
                millivolts: 4317
            })
        );
        assert_eq!(
            parse_reply(&captured_reply(37, 1, 4000)),
            Some(Reading {
                percent: 37,
                charging: Some(true),
                millivolts: 4000
            })
        );
    }

    #[test]
    fn rejects_wrong_or_invalid_reports_and_unknown_charging_is_not_guessed() {
        assert_eq!(
            parse_reply(&captured_reply(51, 9, 3800)).unwrap().charging,
            None
        );
        assert!(parse_reply(&captured_reply(101, 0, 3800)).is_none());
        assert!(parse_reply(&captured_reply(50, 0, 3800)[..16]).is_none());
        let mut bad = captured_reply(50, 0, 3800);
        bad[1] ^= 1;
        assert!(parse_reply(&bad).is_none());
        let mut bad_checksum = captured_reply(50, 0, 3800);
        bad_checksum[16] ^= 1;
        assert!(parse_reply(&bad_checksum).is_none());
    }
}
