//! Read-only ATK/VXE F1 battery transport shared by exact-model adapters.
//! Protocol structure is from Vigil Watt v1.3.0 AtkCodec/atk.md; MIT notice
//! is retained with the distributed package.
use peripheral_desk::{hid::Interface, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_SetOutputReport,
    Foundation::{CloseHandle, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};

const VID: u16 = 0x373b;
const PAGE: u16 = 0xff02;
const USAGE: u16 = 0x0002;
const REPORT_ID: u8 = 0x08;
const OP: u8 = 0x04;
const OP_LEN: usize = 16;
const REPORT_LEN: usize = OP_LEN + 1;

pub fn is_atk_page(i: &Interface) -> bool {
    i.vid == VID && i.usage_page == PAGE && i.usage == USAGE
}

pub fn make_query() -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[0] = REPORT_ID;
    frame[1] = OP;
    frame[16] = checksum(&frame[1..]);
    frame
}

fn checksum(op: &[u8]) -> u8 {
    0x4du8.wrapping_sub(
        op[..OP_LEN - 1]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte)),
    )
}

pub fn parse_reply(frame: &[u8]) -> Option<(u8, Option<bool>, u16)> {
    for offset in [1usize, 0, 2] {
        if frame.len() < offset + OP_LEN {
            continue;
        }
        let op = &frame[offset..offset + OP_LEN];
        if op[0] != OP || op[OP_LEN - 1] != checksum(op) {
            continue;
        }
        let percent = op[5];
        if !(2..=100).contains(&percent) {
            continue;
        }
        let charging = match op[6] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        let voltage = u16::from_be_bytes([op[7], op[8]]);
        return Some((percent, charging, voltage));
    }
    None
}

fn write_with_feature_fallback(path: &str, frame: &[u8]) -> Result<(), String> {
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
            return Err("ATK 查询写入失败，且无法打开 Feature 报告后备通道".into());
        }
        let success = HidD_SetOutputReport(handle, frame.as_ptr(), frame.len() as u32) != 0;
        CloseHandle(handle);
        if success {
            Ok(())
        } else {
            Err("ATK WriteFile 与 HidD_SetOutputReport 查询均未成功".into())
        }
    }
}

pub fn read_state(
    key: &str,
    label: &str,
    interface_match: fn(&Interface) -> bool,
) -> Result<State, String> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    read_state_from(&interfaces, key, label, interface_match, |_| false)
}

pub fn read_state_prefer(
    key: &str,
    label: &str,
    interface_match: fn(&Interface) -> bool,
    preferred: fn(&Interface) -> bool,
) -> Result<State, String> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    read_state_from(&interfaces, key, label, interface_match, preferred)
}

fn read_state_from(
    interfaces: &[Interface],
    key: &str,
    label: &str,
    interface_match: fn(&Interface) -> bool,
    preferred: fn(&Interface) -> bool,
) -> Result<State, String> {
    let candidates: Vec<_> = interfaces.iter().filter(|i| interface_match(i)).collect();
    let preferred_candidates: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|i| preferred(i))
        .collect();
    let candidates = if preferred_candidates.is_empty() {
        candidates
    } else {
        preferred_candidates
    };
    let Some(interface) = candidates.first().copied() else {
        return Ok(State {
            key: key.into(),
            error: format!("未找到 {label} 电量接口"),
            settings_error: "此插件只执行 ATK/VXE 电量只读查询，不提供 DPI、回报率或设置写入。"
                .into(),
            ..State::default()
        });
    };
    if candidates.len() > 1 {
        return Err(format!("发现多个 {label} 匹配接口，停止查询"));
    }
    let mut io = WinTransport::open(&interface.path)?;
    let query = make_query();
    if io.write(&query, Duration::from_millis(500)).is_err() {
        write_with_feature_fallback(&interface.path, &query)?;
    }
    let deadline = std::time::Instant::now() + Duration::from_millis(1800);
    while std::time::Instant::now() < deadline {
        let frame = io.read(
            (interface.input_len as usize).max(REPORT_LEN),
            Duration::from_millis(160),
        )?;
        if let Some((percent, charging, voltage)) = parse_reply(&frame) {
            return Ok(State {
                key: key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging,
                wired: interface.pid == 0x12a9,
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                identity: format!("{label} · 373B:{:04X} · {voltage} mV", interface.pid),
                settings_error: "只读取电量、充电标志和电压；控制功能未由本插件提供。".into(),
                ..State::default()
            });
        }
    }
    Err(format!("{label} 未收到校验通过的电量响应"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_and_response_match_atk_op_frame() {
        let query = make_query();
        assert_eq!(query[0], REPORT_ID);
        assert_eq!(query[1], OP);
        assert_eq!(query[16], 0x49);
        let mut reply = [0; REPORT_LEN];
        reply[0] = REPORT_ID;
        reply[1] = OP;
        reply[6] = 75;
        reply[7] = 1;
        reply[8..10].copy_from_slice(&4322u16.to_be_bytes());
        reply[16] = checksum(&reply[1..]);
        assert_eq!(parse_reply(&reply), Some((75, Some(true), 4322)));
        reply[16] ^= 1;
        assert_eq!(parse_reply(&reply), None);
    }
}
