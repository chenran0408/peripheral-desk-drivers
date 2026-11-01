use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::{
    ptr::null_mut,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_GetInputReport,
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};

fn query(io: &mut WinTransport, path: &str, cmd: u8) -> Result<Vec<u8>, String> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe {
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
    if handle != INVALID_HANDLE_VALUE {
        let mut initial = [0; 64];
        initial[0] = 6;
        unsafe {
            HidD_GetInputReport(handle, initial.as_mut_ptr().cast(), 64);
            CloseHandle(handle);
        }
    }
    let mut request = [0; 62];
    request[..16].copy_from_slice(&[
        6, 0, 2, 0, 0x9a, 0, 0, 0x68, 0x4a, 0x8e, 0x0a, 0, 0, 0, 0xbb, cmd,
    ]);
    io.write(&request, Duration::from_millis(500))?;
    sleep(Duration::from_millis(100));
    let reply = io.read(64, Duration::from_millis(1000))?;
    if reply.len() < 8 || reply[..3] != [0x0b, 0, 0xbb] || reply[3] != cmd {
        return Err("Kingston Cloud II 未回复匹配的电量报告".into());
    }
    Ok(reply)
}

fn read_state() -> Result<State, String> {
    let Some(i) = hid::enumerate()?
        .into_iter()
        .find(|i| i.vid == 0x0951 && i.pid == 0x1718 && i.input_len >= 64 && i.output_len >= 62)
    else {
        return Ok(State {
            key: "hyperx-cloud-ii-wireless-kingston".into(),
            error: "未找到 Kingston 版 Cloud II 接收器".into(),
            ..State::default()
        });
    };
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    let level = query(&mut io, &i.path, 2)?[7];
    if level > 100 {
        return Err("耳机电量超出范围".into());
    }
    let status = query(&mut io, &i.path, 3)?[4];
    Ok(State {
        key: "hyperx-cloud-ii-wireless-kingston".into(),
        present: true,
        fresh: true,
        percent: Some(level),
        charging: Some(status == 1),
        identity: "HyperX Cloud II Wireless (Kingston) · 0951:1718".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅支持只读电量与充电状态。".into(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "hyperx-cloud-ii-wireless-kingston" {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持只读电量".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("hyperx-cloud-ii-wireless-kingston", handler);
