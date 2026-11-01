//! BlackShark V2 Pro 2020 feature report from HaloBattery hardware observations (MIT).
use peripheral_desk::{hid, model::State};
use std::{
    ptr::{null, null_mut},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::{HidD_GetFeature, HidD_SetFeature},
    Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};
pub const KEY: &str = "razer-blackshark-v2-pro-2020";
const LEN: usize = 64;
type Battery = (Option<u8>, Option<bool>);
pub fn request() -> [u8; LEN] {
    let mut r = [0; LEN];
    r[..9].copy_from_slice(&[0xff, 0x0a, 0, 0xfd, 4, 0x12, 0xf1, 2, 5]);
    r
}
pub fn parse(r: &[u8]) -> Option<Battery> {
    if r.len() != LEN || r[..9] != [0xff, 0x0f, 5, 0xfe, 0x12, 4, 0x1f, 8, 5] || r[12..14] == [0, 0]
    {
        return None;
    }
    let charging = match r[11] {
        9 => Some(true),
        1 | 2 | 3 | 6 => Some(false),
        _ => None,
    };
    let level = (r[14] <= 100 && charging != Some(true)).then_some(r[14]);
    Some((level, charging))
}
fn feature(path: &str) -> Result<Option<Battery>, String> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let result = (|| {
        let mut r = request();
        if !unsafe { HidD_SetFeature(handle, r.as_mut_ptr().cast(), LEN as u32) } {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let end = Instant::now() + Duration::from_millis(500);
        loop {
            std::thread::sleep(Duration::from_millis(100));
            let mut reply = [0; LEN];
            reply[0] = 0xff;
            if !unsafe { HidD_GetFeature(handle, reply.as_mut_ptr().cast(), LEN as u32) } {
                return Err(std::io::Error::last_os_error().to_string());
            }
            if let Some(reading) = parse(&reply) {
                return Ok(Some(reading));
            }
            if Instant::now() >= end {
                return Ok(None);
            }
        }
    })();
    unsafe {
        CloseHandle(handle);
    }
    result
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let Some(i) = interfaces
        .iter()
        .find(|i| i.vid == 0x1532 && i.pid == 0x0528 && i.usage_page == 0xff00 && i.usage == 1)
    else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 BlackShark V2 Pro (2020) 接收器".into(),
            ..State::default()
        });
    };
    let (percent, charging) =
        feature(&i.path)?.ok_or("BlackShark 2020 无新电量回复（可能关机）")?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent,
        charging,
        identity: "Razer BlackShark V2 Pro (2020) · 1532:0528".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "充电期间设备上报电量不准确，仅显示充电状态。".into(),
        ..State::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_feature_only() {
        let mut r = [0; LEN];
        r[..9].copy_from_slice(&[0xff, 0x0f, 5, 0xfe, 0x12, 4, 0x1f, 8, 5]);
        r[12] = 0x10;
        r[14] = 80;
        r[11] = 9;
        assert_eq!(parse(&r), Some((None, Some(true))));
        r[11] = 1;
        assert_eq!(parse(&r), Some((Some(80), Some(false))));
        r[1] = 1;
        assert_eq!(parse(&r), None);
    }
}
