//! Sony INZONE H9 II (WH-G900N2) receiver battery/charging query.
//! Adapted from Vigil Watt v1.3.0, commit a78e973b4eb634bdce37c9078dd48d1c07e3eb73.
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
    Devices::HumanInterfaceDevice::HidD_SetOutputReport,
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_WRITE, OPEN_EXISTING,
    },
};

pub const DEVICE_KEY: &str = "sony-inzone-h9-ii";
const VID: u16 = 0x054C;
const PID: u16 = 0x0FA8;
const USAGE_PAGE: u16 = 0xFF04;
const USAGE: u16 = 1;
const RID: u8 = 0x02;
const LEN: usize = 64;

pub fn request_frame(class: u8, command: u8) -> Option<[u8; LEN]> {
    if class != 0x41 || command != 0x04 {
        return None;
    }
    let mut f = [0; LEN];
    f[0] = RID;
    f[1] = 0x0C;
    f[2] = 1;
    f[4] = 0xFC;
    f[5] = 8;
    f[6] = 0x96;
    f[7] = 0xC3;
    f[8] = class;
    f[9] = command;
    f[10] = 1;
    f[11] = 1;
    f[13] = 0x9C + command;
    Some(f)
}
pub fn parse_battery(f: &[u8]) -> Option<(u8, bool)> {
    if f.len() < LEN
        || f[0] != RID
        || f[8] != 0x14
        || f[9] != 0x04
        || !matches!(f[10], 0x10 | 0xA0)
        || f[14] > 100
        || f[13] > 1
    {
        return None;
    }
    Some((f[14], f[13] == 1))
}
fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == LEN
        && i.output_len as usize == LEN
        && i.path.to_ascii_lowercase().contains("&mi_05&col03#")
}

struct OutputHandle(HANDLE);
impl Drop for OutputHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
impl OutputHandle {
    fn open(path: &str) -> Result<Self, String> {
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let h = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if h == INVALID_HANDLE_VALUE {
            Err(format!(
                "无法打开 INZONE 输出通道：{}",
                std::io::Error::last_os_error()
            ))
        } else {
            Ok(Self(h))
        }
    }
    fn send_control_report(&self, frame: &[u8; LEN]) -> Result<(), String> {
        let ok =
            unsafe { HidD_SetOutputReport(self.0, frame.as_ptr().cast::<c_void>(), LEN as u32) };
        if !ok {
            Err(format!(
                "INZONE SET_REPORT 查询发送失败：{}",
                std::io::Error::last_os_error()
            ))
        } else {
            Ok(())
        }
    }
}

fn query(path: &str) -> Result<(u8, bool), String> {
    let output = OutputHandle::open(path)?;
    let mut input = WinTransport::open(path)?;
    // This headset echoes interrupt OUT writes; only control SET_REPORT is valid.
    output.send_control_report(&request_frame(0x41, 0x04).ok_or("未知 INZONE 查询")?)?;
    let deadline = Instant::now() + Duration::from_millis(750);
    while Instant::now() < deadline {
        let f = input.read(LEN, Duration::from_millis(250))?;
        if f.is_empty() {
            continue;
        }
        if let Some(result) = parse_battery(&f) {
            return Ok(result);
        }
    }
    Err("INZONE H9 II 未收到匹配的电量响应".into())
}
pub fn read_state() -> Result<State, String> {
    let items = hid::enumerate()?;
    let mut matches = items.iter().filter(|i| supported(i));
    let Some(i) = matches.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "未找到 INZONE H9 II 接收器的 FF04 控制接口".into(),
            settings_error: "仅适配接收器模式的只读电量与充电查询；音效、EQ 和设置写入未开放。"
                .into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err("发现多个 INZONE H9 II 接收器接口，已停止查询".into());
    }
    let (percent, charging) = query(&i.path)?;
    Ok(State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: Some(charging),
        wired: false,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: "Sony INZONE H9 II · 接收器 · 054C:0FA8 · FF04".into(),
        settings_error: "只读电量和充电状态；EQ、音量、灯效和其他设置未适配。".into(),
        ..State::default()
    })
}
