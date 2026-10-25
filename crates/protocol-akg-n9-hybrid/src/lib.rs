//! AKG N9 Hybrid exact receiver battery query, adapted from Vigil Watt v1.3.0.
//! Source commit a78e973b4eb634bdce37c9078dd48d1c07e3eb73; see VIGIL-WATT-MIT.txt.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::{
    ffi::c_void,
    ptr::null_mut,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_GetInputReport,
    Foundation::{CloseHandle, GENERIC_READ, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};

pub const DEVICE_KEY: &str = "akg-n9-hybrid";
pub const VID: u16 = 0x0ECB;
pub const PID: u16 = 0x20D9;
const USAGE_PAGE: u16 = 0xFF13;
const USAGE: u16 = 1;
const REPORT_LEN: usize = 62;
const IN_REPORT_ID: u8 = 0x07;
const OUT_REPORT_ID: u8 = 0x06;
const MAX_PAYLOAD: usize = 59;

/// Fixed output report. This receiver requires the exact vendor-interface OUT report.
pub fn query() -> [u8; REPORT_LEN] {
    let mut r = [0; REPORT_LEN];
    r[..10].copy_from_slice(&[
        OUT_REPORT_ID,
        0x07,
        0x80,
        0x05,
        0x5A,
        0x03,
        0x00,
        0xD6,
        0x0C,
        0x00,
    ]);
    r
}

/// Parse only a complete N9 battery result message. The payload byte is raw percent.
pub fn parse_message(message: &[u8]) -> Option<u8> {
    (message.len() == 9
        && message[0] == 5
        && message[1] == 0x5D
        && message[2] == 5
        && message[3] == 0
        && message[4] == 0xD6
        && message[5] == 0x0C
        && message[6] == 0
        && message[7] == 0
        && message[8] <= 100)
        .then_some(message[8])
}

fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.feature_len == 0
        && i.path.to_ascii_lowercase().contains("&mi_03&col02#")
}

struct InputHandle(HANDLE);
impl Drop for InputHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
impl InputHandle {
    fn open(path: &str) -> Result<Self, String> {
        let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let h = unsafe {
            CreateFileW(
                path.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if h == INVALID_HANDLE_VALUE {
            Err(format!(
                "无法打开 AKG 输入报告：{}",
                std::io::Error::last_os_error()
            ))
        } else {
            Ok(Self(h))
        }
    }
    fn read(&self) -> Option<[u8; REPORT_LEN]> {
        let mut report = [0; REPORT_LEN];
        report[0] = IN_REPORT_ID;
        let ok = unsafe {
            HidD_GetInputReport(
                self.0,
                report.as_mut_ptr().cast::<c_void>(),
                REPORT_LEN as u32,
            )
        };
        ok.then_some(report)
    }
}

fn payload(report: &[u8]) -> Option<&[u8]> {
    (report.len() == REPORT_LEN
        && report[0] == IN_REPORT_ID
        && report[2] == 0x80
        && (report[1] as usize) <= MAX_PAYLOAD)
        .then(|| &report[3..3 + report[1] as usize])
}

fn query_one(path: &str) -> Result<u8, String> {
    let input = InputHandle::open(path)?;
    let mut transport = WinTransport::open(path)?;
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(report) = input.read() {
            if payload(&report).is_some_and(|p| p.is_empty()) {
                break;
            }
        }
        if Instant::now() >= deadline {
            return Err("AKG 输入接口未进入空闲态，已停止查询".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    transport.write(&query(), Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut pending = Vec::new();
    let mut ack = false;
    let mut previous: Option<Vec<u8>> = None;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
        let Some(report) = input.read() else { continue };
        let Some(fragment) = payload(&report) else {
            return Err("AKG 输入报告格式不匹配".into());
        };
        if fragment.is_empty() {
            previous = None;
            continue;
        }
        let mut snapshot = report[..3 + fragment.len()].to_vec();
        if previous.as_ref() == Some(&snapshot) {
            continue;
        }
        previous = Some(std::mem::take(&mut snapshot));
        pending.extend_from_slice(fragment);
        loop {
            if pending.is_empty() {
                break;
            }
            if pending[0] != 5 {
                return Err("AKG 返回未知消息头".into());
            }
            if pending.len() < 4 {
                break;
            }
            if !matches!(pending[1], 0x5B..=0x5D) {
                return Err("AKG 返回未知消息类型".into());
            }
            let size = 4 + pending[2] as usize + ((pending[3] as usize) << 8);
            if !(6..=1024).contains(&size) {
                return Err("AKG 返回消息长度无效".into());
            }
            if pending.len() < size {
                break;
            }
            let message: Vec<u8> = pending.drain(..size).collect();
            if message.len() >= 6 && message[4] == 0xD6 && message[5] == 0x0C {
                if message[1] == 0x5B {
                    if message.len() != 7 || message[6] != 0 {
                        return Err("AKG 查询确认失败".into());
                    }
                    ack = true;
                } else if message[1] == 0x5D && ack {
                    return parse_message(&message)
                        .ok_or_else(|| "AKG 电量字段格式或范围无效".into());
                }
            }
        }
    }
    Err("AKG 查询未收到确认后的电量结果".into())
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut matching = interfaces.iter().filter(|i| supported(i));
    let Some(i) = matching.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "未找到 AKG N9 Hybrid 指定接收器接口".into(),
            settings_error: "只读电量查询；充电状态和其他控制尚未适配。".into(),
            ..State::default()
        });
    };
    if matching.next().is_some() {
        return Err("发现多个 AKG N9 电量接口，已停止查询".into());
    }
    let percent = query_one(&i.path)?;
    Ok(State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: None,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: "AKG N9 Hybrid · USB 接收器 · 0ECB:20D9 · FF13".into(),
        settings_error: "只读电量；充电状态、均衡器和其他控制未适配。".into(),
        ..State::default()
    })
}
