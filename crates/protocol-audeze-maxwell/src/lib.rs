//! Audeze Maxwell exchange from HaloBattery providers/audeze.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::{
    ptr::null_mut,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_GetInputReport,
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};

pub const KEY: &str = "audeze-maxwell";
pub const MAXWELL2_KEY: &str = "audeze-maxwell2";
const LEN: usize = 62;
const BATTERY: &[u8] = &[6, 7, 0x80, 5, 0x5a, 3, 0, 0xd6, 0x0c];
const INIT: &[&[u8]] = &[
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x20],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x25],
    &[6, 7, 0x80, 5, 0x5a, 3, 0, 7, 0x1c],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x28],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 0x83, 0x2c, 1],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 0x83, 0x2c, 7],
    &[6, 7, 0, 5, 0x5a, 3, 0, 7, 0x1c],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x2d],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x2c],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 0x83, 0x2c, 0x0b],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x24],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x2f],
    BATTERY,
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x22],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 0x83, 0x2c, 0x0b],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x24],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 1, 9, 0x2c],
    &[6, 8, 0x80, 5, 0x5a, 4, 0, 0x83, 0x2c, 7],
];

pub fn request(prefix: &[u8]) -> [u8; LEN] {
    let mut data = [0; LEN];
    data[..prefix.len()].copy_from_slice(prefix);
    data
}

pub fn parse(reply: &[u8]) -> Option<u8> {
    if reply.first() != Some(&7) && !reply.starts_with(&[5, 0x5d]) {
        return None;
    }
    reply
        .windows(5)
        .find_map(|w| (w[..4] == [0xd6, 0x0c, 0, 0] && w[4] <= 100).then_some(w[4]))
        .or_else(|| {
            reply.windows(4).enumerate().find_map(|(offset, header)| {
                (header[..2] == [5, 0x5d])
                    .then(|| {
                        reply[offset + 4..]
                            .windows(3)
                            .find_map(|w| (w[..2] == [0xd6, 0x0c] && w[2] <= 100).then_some(w[2]))
                    })
                    .flatten()
            })
        })
        .or_else(|| {
            reply.windows(3).enumerate().find_map(|(offset, w)| {
                (w[..2] == [0xd6, 0x0c]
                    && reply.get(offset + 2..offset + 4) != Some(&[0, 0])
                    && w[2] <= 100)
                    .then_some(w[2])
            })
        })
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
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let raw = unsafe {
            CreateFileW(
                wide.as_ptr(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            Ok(Self(raw))
        }
    }
    fn read(&self) -> Option<[u8; LEN]> {
        let mut reply = [0; LEN];
        reply[0] = 7;
        unsafe { HidD_GetInputReport(self.0, reply.as_mut_ptr().cast(), LEN as u32) }
            .then_some(reply)
    }
}

fn sequence(
    io: &mut WinTransport,
    input: &InputHandle,
    packets: &[&[u8]],
) -> Result<Option<u8>, String> {
    let mut newest = None;
    for packet in packets {
        io.write(&request(packet), Duration::from_millis(500))?;
        sleep(Duration::from_millis(60));
        if let Some(reply) = input.read() {
            newest = parse(&reply).or(newest);
        }
    }
    for _ in 0..2 {
        sleep(Duration::from_millis(60));
        if let Some(reply) = input.read() {
            newest = parse(&reply).or(newest);
        }
    }
    Ok(newest)
}

pub fn read_state() -> Result<State, String> {
    read_state_for(KEY, &[0x4b19, 0x4b1a])
}

pub fn read_state_for(key: &str, pids: &[u16]) -> Result<State, String> {
    let mut interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| {
            i.vid == 0x3329 && pids.contains(&i.pid) && i.usage_page == 0xff13 && i.usage == 1
        })
        .collect();
    interfaces.sort_by_key(|i| i.pid != 0x4b1a);
    if interfaces.is_empty() {
        return Ok(State {
            key: key.into(),
            error: "未找到 Maxwell 控制接口".into(),
            ..State::default()
        });
    }
    for i in &interfaces {
        if i.pid == 0x4b19
            && i.product
                .trim()
                .eq_ignore_ascii_case("Audeze Maxwell Dongle")
        {
            continue;
        }
        let (Ok(mut io), Ok(input)) = (WinTransport::open(&i.path), InputHandle::open(&i.path))
        else {
            continue;
        };
        let percent = match sequence(&mut io, &input, &[BATTERY])? {
            found @ Some(_) => found,
            None => sequence(&mut io, &input, INIT)?,
        };
        if let Some(percent) = percent {
            return Ok(State {
                key: key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: (key == KEY).then_some(i.pid == 0x4b1a),
                wired: key == KEY && i.pid == 0x4b1a,
                identity: format!(
                    "{} · 3329:{:04X}",
                    if key == KEY {
                        "Audeze Maxwell"
                    } else {
                        "Audeze Maxwell 2"
                    },
                    i.pid
                ),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: if key == KEY {
                    "仅支持只读电量；USB 连接推断充电。"
                } else {
                    "仅支持只读电量；未验证充电状态。"
                }
                .into(),
                ..State::default()
            });
        }
    }
    Ok(State {
        key: key.into(),
        present: true,
        error: "Maxwell 电量未知（耳机可能未连接）".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker() {
        assert_eq!(&request(BATTERY)[..9], BATTERY);
        assert_eq!(parse(&[7, 0, 0xd6, 0x0c, 0, 0, 76]), Some(76));
        assert_eq!(parse(&[7, 5, 0x5d, 7, 0, 0xd6, 0x0c, 75]), Some(75));
        assert_eq!(parse(&[7, 5, 0x5d, 7, 0, 0xd6, 0x0c, 0, 0]), Some(0));
        assert_eq!(parse(&[5, 0x5d, 7, 0, 0xd6, 0x0c, 75]), Some(75));
        assert_eq!(parse(&[7, 0, 0xd6, 0x0c, 0, 0, 255]), None);
    }
}
