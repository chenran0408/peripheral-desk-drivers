//! Read-only client for the Flydigi Space Station local device list.
//!
//! The framing and protobuf schema follow BatteryMonitor (MIT, Copyright (c)
//! 2026 JKWTCN), commit 25c221326391e26b62ae2b7db8e72ce8538ff909.
//! https://github.com/JKWTCN/BatteryMonitor/blob/25c221326391e26b62ae2b7db8e72ce8538ff909/src/providers/xbox/FlydigiControllerReader.cpp

use super::Reading;
use std::{
    mem::zeroed,
    ptr::{null, null_mut},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, ERROR_FILE_NOT_FOUND, ERROR_IO_PENDING, ERROR_MORE_DATA,
        ERROR_PIPE_BUSY, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
    },
    Storage::FileSystem::{
        CreateFileW, ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, OPEN_EXISTING,
        SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
    },
    System::{
        Threading::{CreateEventW, WaitForSingleObject},
        IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
    },
};

const HEADER: &[u8] = b"FDG_PROTOCOL\n";
const DEVICE_LIST: u64 = 4097;
const DEVICE_CHANGED: u64 = 4098;
const REQUEST: &[u8] = b"FDG_PROTOCOL\n\x05\x00\x00\x00\x08\x81\x20\x1a\x00";
const MAX_PAYLOAD: usize = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_millis(1500);

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

impl Handle {
    fn open() -> Result<Self, String> {
        let path: Vec<u16> = r"\\.\pipe\fcs.sock".encode_utf16().chain(Some(0)).collect();
        let pipe = unsafe {
            CreateFileW(
                path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                null(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED | SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
                null_mut(),
            )
        };
        if pipe != INVALID_HANDLE_VALUE {
            return Ok(Self(pipe));
        }
        match unsafe { GetLastError() } {
            ERROR_FILE_NOT_FOUND => Err("未找到飞智空间站后台服务，请先安装飞智空间站。".into()),
            ERROR_PIPE_BUSY => {
                Err("飞智空间站的电量通道被占用，请从托盘退出空间站界面，保留后台服务。".into())
            }
            error => Err(format!(
                "无法连接飞智空间站后台服务：{}",
                std::io::Error::from_raw_os_error(error as i32)
            )),
        }
    }

    fn transfer(&self, bytes: &mut [u8], write: bool, deadline: Instant) -> Result<usize, String> {
        let timeout = deadline
            .checked_duration_since(Instant::now())
            .ok_or("飞智空间站电量查询超时。")?;
        unsafe {
            let event = CreateEventW(null(), 1, 0, null());
            if event.is_null() {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let event = Handle(event);
            let mut overlapped: OVERLAPPED = zeroed();
            overlapped.hEvent = event.0;
            let mut count = 0;
            let ok = if write {
                WriteFile(
                    self.0,
                    bytes.as_ptr(),
                    bytes.len() as u32,
                    &mut count,
                    &mut overlapped,
                )
            } else {
                ReadFile(
                    self.0,
                    bytes.as_mut_ptr(),
                    bytes.len() as u32,
                    &mut count,
                    &mut overlapped,
                )
            };
            if ok == 0 {
                match GetLastError() {
                    ERROR_IO_PENDING => {
                        if WaitForSingleObject(event.0, timeout.as_millis().max(1) as u32)
                            != WAIT_OBJECT_0
                        {
                            CancelIoEx(self.0, &overlapped);
                            // Drain this operation before its stack buffer and event go away.
                            GetOverlappedResult(self.0, &overlapped, &mut count, 1);
                            return Err("飞智空间站电量查询超时。".into());
                        }
                        if GetOverlappedResult(self.0, &overlapped, &mut count, 0) == 0
                            && (write || GetLastError() != ERROR_MORE_DATA)
                        {
                            return Err(std::io::Error::last_os_error().to_string());
                        }
                    }
                    ERROR_MORE_DATA if !write => {}
                    _ => return Err(std::io::Error::last_os_error().to_string()),
                }
            }
            Ok(count as usize)
        }
    }

    fn read_exact(&self, mut bytes: &mut [u8], deadline: Instant) -> Result<(), String> {
        while !bytes.is_empty() {
            let count = self.transfer(bytes, false, deadline)?;
            if count == 0 {
                return Err("飞智空间站提前关闭了电量查询连接。".into());
            }
            bytes = &mut bytes[count..];
        }
        Ok(())
    }
}

fn read_frame(
    mut read_exact: impl FnMut(&mut [u8]) -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    let mut header = [0; HEADER.len()];
    read_exact(&mut header)?;
    if header != HEADER {
        return Err("飞智空间站电量回复的帧头无效。".into());
    }
    let mut length = [0; 4];
    read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length > MAX_PAYLOAD {
        return Err("飞智空间站电量回复过长。".into());
    }
    let mut payload = vec![0; length];
    read_exact(&mut payload)?;
    Ok(payload)
}

#[derive(Clone, Copy)]
enum Field<'a> {
    Integer(u64),
    Bytes(&'a [u8]),
    Other,
}

fn varint(input: &mut &[u8]) -> Option<u64> {
    let mut value = 0;
    for shift in (0..64).step_by(7) {
        let (&byte, remaining) = input.split_first()?;
        *input = remaining;
        if shift == 63 && byte > 1 {
            return None;
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte < 0x80 {
            return Some(value);
        }
    }
    None
}

fn take<'a>(input: &mut &'a [u8], len: usize) -> Option<&'a [u8]> {
    let (chunk, remaining) = input.split_at_checked(len)?;
    *input = remaining;
    Some(chunk)
}

fn visit_fields<'a>(mut input: &'a [u8], mut visit: impl FnMut(u64, Field<'a>)) -> Option<()> {
    while !input.is_empty() {
        let tag = varint(&mut input)?;
        let field = match tag & 7 {
            0 => Field::Integer(varint(&mut input)?),
            1 => {
                take(&mut input, 8)?;
                Field::Other
            }
            2 => {
                let length = usize::try_from(varint(&mut input)?).ok()?;
                Field::Bytes(take(&mut input, length)?)
            }
            5 => {
                take(&mut input, 4)?;
                Field::Other
            }
            _ => return None,
        };
        visit(tag >> 3, field);
    }
    Some(())
}

#[derive(Default)]
struct Device {
    name: String,
    category: u64,
    link: u64,
    code: String,
    device_type: u64,
    identity: Vec<u8>,
    connected: bool,
    level: Option<u8>,
    vid: u64,
    pid: u64,
}

impl Device {
    fn parse(payload: &[u8]) -> Option<Self> {
        let mut device = Self::default();
        visit_fields(payload, |number, field| match (number, field) {
            (1, Field::Bytes(value)) => {
                device.name = String::from_utf8_lossy(value).into_owned();
            }
            (2, Field::Integer(value)) => device.category = value,
            (3, Field::Integer(value)) => device.link = value,
            (4, Field::Bytes(value)) => device.code = String::from_utf8_lossy(value).into_owned(),
            (5, Field::Integer(value)) => device.device_type = value,
            (6, Field::Bytes(value)) if device.identity.is_empty() => {
                device.identity = value.to_vec()
            }
            (7, Field::Bytes(value)) if !value.is_empty() => device.identity = value.to_vec(),
            (8, Field::Integer(value)) => device.connected = value == 1,
            (9, Field::Integer(value)) => device.level = u8::try_from(value).ok(),
            (11, Field::Integer(value)) => device.vid = value,
            (12, Field::Integer(value)) => device.pid = value,
            _ => {}
        })?;
        Some(device)
    }

    fn model_matches(&self) -> bool {
        if !matches!(self.category, 0 | 1) {
            return false;
        }
        let normalized = self
            .name
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase();
        let name = normalized
            .strip_prefix("flydigi")
            .or_else(|| normalized.strip_prefix("飞智"))
            .unwrap_or(&normalized);
        let name = name.strip_suffix("wireless").unwrap_or(name);
        self.code.eq_ignore_ascii_case("f4")
            || matches!(name, "黑武士4pro" | "vader4pro")
            || matches!(
                (name, self.device_type),
                ("黑武士4" | "vader4" | "", 85 | 91 | 105)
            )
    }

    fn reading(&self) -> Reading {
        let (percent, charging) = match self.level {
            // The service reports 0 for a nearly empty controller, unlike the
            // zero-filled HID reply during receiver initialization.
            Some(0) => (Some(5), Some(false)),
            Some(level @ 1..=5) => (Some(20 * level), Some(false)),
            Some(6) => (None, Some(true)),
            _ => (None, None),
        };
        Reading {
            raw: self.level,
            percent,
            charging,
            wired: match self.link {
                1 => Some(true),
                2 => Some(false),
                _ => None,
            },
        }
    }
}

fn parse_reply(payload: &[u8]) -> Option<(u64, u64, Vec<Device>)> {
    let mut command = 0;
    let mut result = 0;
    let mut list = None;
    let mut changed = None;
    visit_fields(payload, |number, field| match (number, field) {
        (1, Field::Integer(id)) => command = id,
        (3, Field::Integer(code)) => result = code,
        (5, Field::Bytes(container)) => list = Some(container),
        (6, Field::Bytes(container)) => changed = Some(container),
        _ => {}
    })?;
    let container = match command {
        DEVICE_LIST => Some(list.unwrap_or_default()),
        DEVICE_CHANGED => changed,
        _ => return None,
    }?;
    let mut devices = Vec::new();
    visit_fields(container, |number, field| {
        if let (1, Field::Bytes(bytes)) = (number, field) {
            if let Some(device) = Device::parse(bytes).filter(Device::model_matches) {
                devices.push(device);
            }
        }
    })?;
    Some((command, result, devices))
}

pub(super) fn query_vader4(require_xinput_id: bool) -> Result<Reading, String> {
    let pipe = Handle::open()?;
    let deadline = Instant::now() + TIMEOUT;
    if pipe.transfer(&mut REQUEST.to_vec(), true, deadline)? != REQUEST.len() {
        return Err("未能完整发送飞智空间站的电量查询。".into());
    }
    let mut events: Vec<Device> = Vec::new();
    loop {
        let frame = read_frame(|buffer| pipe.read_exact(buffer, deadline))?;
        let Some((command, result, devices)) = parse_reply(&frame) else {
            continue;
        };
        if result != 0 {
            return Err(format!("飞智空间站返回电量查询错误 {result}。"));
        }
        if command == DEVICE_CHANGED {
            // Without a stable controller identity an event could be a duplicate.
            for device in devices
                .into_iter()
                .filter(|device| !device.identity.is_empty())
            {
                events.retain(|previous| previous.identity != device.identity);
                events.push(device);
            }
            continue;
        }
        let mut candidates = if devices.is_empty() { events } else { devices }
            .into_iter()
            .filter(|device| {
                device.connected
                    && matches!(device.link, 1 | 2)
                    && ((device.vid, device.pid) == (0x045e, 0x028e)
                        || (!require_xinput_id && (device.vid, device.pid) == (0, 0)))
            });
        let device = candidates
            .next()
            .ok_or("飞智空间站未报告在线的黑武士 4 Pro。")?;
        if candidates.next().is_some() {
            return Err("飞智空间站报告多台黑武士 4 Pro，无法确认电量属于哪台。".into());
        }
        return Ok(device.reading());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    fn frame(payload: &[u8]) -> Vec<u8> {
        [HEADER, &(payload.len() as u32).to_le_bytes(), payload].concat()
    }

    #[test]
    fn reads_battery_from_the_matching_online_device_list_entry() {
        // Representative device list response. The payload contains another model
        // at full charge and our wireless Vader 4 Pro at level 3.
        let payload = b"\x08\x81\x20\x2a\x2a\x0a\x11\x0a\x0bVader 5 Pro\x40\x01\x48\x05\x0a\x15\x0a\x0bVader 4 Pro\x18\x02\x40\x01\x48\x03\x58\x01";
        let mut bytes = Cursor::new(frame(payload));
        let reply =
            read_frame(|buffer| bytes.read_exact(buffer).map_err(|error| error.to_string()))
                .unwrap();
        let (command, result, devices) = parse_reply(&reply).unwrap();
        assert_eq!(command, DEVICE_LIST);
        assert_eq!(result, 0);
        assert_eq!(devices.len(), 1);
        assert!(devices[0].connected);
        assert_eq!(
            devices[0].reading(),
            Reading {
                raw: Some(3),
                percent: Some(60),
                charging: Some(false),
                wired: Some(false),
            }
        );
    }

    #[test]
    fn separates_charging_status_from_unreported_level() {
        let mut device = Device::parse(b"\x0a\x0bVader 4 Pro\x40\x01").unwrap();
        assert_eq!(device.reading().percent, None);
        assert_eq!(device.reading().charging, None);
        device.level = Some(6);
        assert_eq!(device.reading().percent, None);
        assert_eq!(device.reading().charging, Some(true));
    }
}
