//! Bounded Windows interrupt-report I/O shared by device-specific protocols.
type Result<T> = std::result::Result<T, String>;
use std::{
    mem::zeroed,
    ptr::{null, null_mut},
    time::Duration,
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_FlushQueue,
    Foundation::*,
    Storage::FileSystem::*,
    System::{Threading::*, IO::*},
};
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
pub struct WinTransport {
    reader: Handle,
    writer: Handle,
}
impl WinTransport {
    pub fn open(path: &str) -> Result<Self> {
        fn open(path: &str, access: u32) -> Result<Handle> {
            let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
            let h = unsafe {
                CreateFileW(
                    wide.as_ptr(),
                    access,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    null(),
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    null_mut(),
                )
            };
            if h == INVALID_HANDLE_VALUE {
                Err(format!(
                    "打开 HID 通道失败：{}",
                    std::io::Error::last_os_error()
                ))
            } else {
                Ok(Handle(h))
            }
        }
        Ok(Self {
            reader: open(path, GENERIC_READ)?,
            writer: open(path, GENERIC_WRITE)?,
        })
    }
    fn transfer(handle: &Handle, bytes: &mut [u8], write: bool, timeout: u32) -> Result<usize> {
        unsafe {
            let event = Handle(CreateEventW(null(), 1, 0, null()));
            if event.0.is_null() {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let mut ov: OVERLAPPED = zeroed();
            ov.hEvent = event.0;
            let mut count = 0;
            let ok = if write {
                WriteFile(
                    handle.0,
                    bytes.as_ptr(),
                    bytes.len() as u32,
                    &mut count,
                    &mut ov,
                )
            } else {
                ReadFile(
                    handle.0,
                    bytes.as_mut_ptr(),
                    bytes.len() as u32,
                    &mut count,
                    &mut ov,
                )
            };
            if ok == 0 {
                if GetLastError() != ERROR_IO_PENDING {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                if WaitForSingleObject(event.0, timeout) != WAIT_OBJECT_0 {
                    CancelIoEx(handle.0, &ov);
                    // Drain cancellation before OVERLAPPED, event and buffer go out of scope.
                    GetOverlappedResult(handle.0, &ov, &mut count, 1);
                    return if write {
                        Err("HID 查询发送超时".into())
                    } else {
                        Ok(0)
                    };
                }
                if GetOverlappedResult(handle.0, &ov, &mut count, 0) == 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
            }
            Ok(count as usize)
        }
    }
}
impl WinTransport {
    pub fn flush(&mut self) -> Result<()> {
        if !unsafe { HidD_FlushQueue(self.reader.0) } {
            return Err(format!(
                "清理 HID 旧回复失败：{}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
    pub fn write(&mut self, request: &[u8], timeout: Duration) -> Result<()> {
        if request.is_empty() || request.len() > 4096 {
            return Err("HID 报告长度无效".into());
        }
        let mut wire = request.to_vec();
        if Self::transfer(&self.writer, &mut wire, true, timeout_ms(timeout))? != request.len() {
            return Err("HID 查询未完整发送".into());
        }
        Ok(())
    }
    pub fn read(&mut self, input_len: usize, timeout: Duration) -> Result<Vec<u8>> {
        if input_len == 0 || input_len > 4096 {
            return Err("HID 报告长度无效".into());
        }
        let mut reply = vec![0; input_len];
        let count = Self::transfer(&self.reader, &mut reply, false, timeout_ms(timeout))?;
        reply.truncate(count);
        Ok(reply)
    }
}
fn timeout_ms(timeout: Duration) -> u32 {
    timeout.as_millis().min(30_000) as u32
}
