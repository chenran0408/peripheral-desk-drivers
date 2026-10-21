//! ABI 1 glue. Each DLL owns every response allocation and frees it itself.
pub use peripheral_desk::plugin_entry::request as core_request;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    slice, str,
};

#[repr(C)]
pub struct Buffer {
    pub ptr: *mut u8,
    pub len: usize,
}
impl Buffer {
    fn from_text(text: String) -> Self {
        let mut bytes = text.into_bytes().into_boxed_slice();
        let result = Self {
            ptr: bytes.as_mut_ptr(),
            len: bytes.len(),
        };
        std::mem::forget(bytes);
        result
    }
}
fn failure(device_key: &str, error: &str) -> String {
    serde_json::json!({"ok": false, "device_key": device_key, "value": null, "error": error})
        .to_string()
}
/// The host must supply a valid, readable input region until this call returns.
/// No input pointer is retained. JSON responses are UTF-8, without a terminator.
///
/// # Safety
/// A non-null input must point to `len` readable bytes; null is accepted only
/// with zero length. Host and DLL must use the same pointer width (x64).
pub unsafe fn request(device_key: &str, input: *const u8, len: usize) -> Buffer {
    request_with(
        device_key,
        input,
        len,
        peripheral_desk::plugin_entry::request,
    )
}
/// A new device may supply its own protocol handler without changing the ABI.
///
/// # Safety
/// The pointer validity contract is the same as [`request`].
pub unsafe fn request_with(
    device_key: &str,
    input: *const u8,
    len: usize,
    handler: fn(&str, &str) -> String,
) -> Buffer {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if len > 1024 * 1024 {
            return failure(device_key, "驱动请求超过 1 MiB");
        }
        if input.is_null() && len != 0 {
            return failure(device_key, "驱动请求指针为空");
        }
        let bytes = if len == 0 {
            &[]
        } else {
            slice::from_raw_parts(input, len)
        };
        let text = match str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return failure(device_key, "驱动请求不是有效的 UTF-8"),
        };
        handler(device_key, text)
    }));
    Buffer::from_text(
        result.unwrap_or_else(|_| failure(device_key, "设备驱动发生异常，本次请求未完成")),
    )
}
/// # Safety
/// Only pass the unchanged pointer and length returned by this same DLL's
/// request function, exactly once. Never pass the buffer to another allocator.
pub unsafe fn free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
    }
}

#[macro_export]
macro_rules! export_driver {
    ($key:literal) => {
        $crate::export_driver!($key, $crate::core_request);
    };
    ($key:literal, $handler:path) => {
        #[no_mangle]
        pub extern "C" fn pd_driver_abi() -> u32 {
            1
        }
        /// # Safety
        /// The host must provide a readable UTF-8 request buffer.
        #[no_mangle]
        pub unsafe extern "C" fn pd_driver_request(input: *const u8, len: usize) -> $crate::Buffer {
            $crate::request_with($key, input, len, $handler)
        }
        /// # Safety
        /// Free only a response returned by this DLL, once, with its exact length.
        #[no_mangle]
        pub unsafe extern "C" fn pd_driver_free(ptr: *mut u8, len: usize) {
            $crate::free(ptr, len);
        }
    };
}
