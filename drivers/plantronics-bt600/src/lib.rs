use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::{
    ptr::null_mut,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::HidD_SetOutputReport,
    Foundation::{CloseHandle, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
};

fn write_report(path: &str, frame: &[u8]) -> Result<(), String> {
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
        return Err(std::io::Error::last_os_error().to_string());
    }
    let ok = unsafe { HidD_SetOutputReport(h, frame.as_ptr().cast(), frame.len() as u32) };
    unsafe {
        CloseHandle(h);
    }
    if !ok {
        return Err(format!(
            "BT600 输出查询失败：{}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn frame(op: u8, group: u8, item: u8, addr: u8) -> [u8; 64] {
    let mut out = [0; 64];
    out[..6].copy_from_slice(&[7, 1, 1, 0x10, if op == 2 { 6 } else { 7 }, addr]);
    out[8..11].copy_from_slice(&[op, group, item]);
    out
}

fn reply(io: &mut WinTransport, op: u8, group: u8, item: u8, addr: u8) -> Result<Vec<u8>, String> {
    for _ in 0..16 {
        let r = io.read(64, Duration::from_millis(250))?;
        if r.len() >= 16
            && r[..4] == [7, 1, 1, 0x10]
            && r[5] == addr
            && r[8..11] == [op, group, item]
        {
            return Ok(r);
        }
    }
    Err("BT600 耳机未连接或未回复".into())
}

fn read_state() -> Result<State, String> {
    let Some(i) = hid::enumerate()?.into_iter().find(|i| {
        i.vid == 0x047f
            && i.pid == 0x02ee
            && i.usage_page == 0xffa2
            && i.usage == 3
            && i.path.to_ascii_lowercase().contains("mi_03")
            && i.input_len >= 64
    }) else {
        return Ok(State {
            key: "plantronics-bt600".into(),
            error: "未找到 Plantronics BT600 接收器".into(),
            ..State::default()
        });
    };
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    write_report(&i.path, &[6, 1])?;
    write_report(&i.path, &frame(1, 1, 2, 0))?;
    reply(&mut io, 8, 1, 2, 0)?;
    write_report(&i.path, &frame(1, 1, 2, 0x20))?;
    reply(&mut io, 8, 1, 2, 2)?;
    write_report(&i.path, &frame(2, 0x0a, 0x1a, 0x20))?;
    let r = reply(&mut io, 3, 0x0a, 0x1a, 2)?;
    let count = r[12];
    if count < 2 || r[11] >= count {
        return Err("BT600 电量档位无效".into());
    }
    Ok(State {
        key: "plantronics-bt600".into(),
        present: true,
        fresh: true,
        percent: Some(((r[11] as u16 * 100) / (count as u16 - 1)) as u8),
        charging: Some(r[13] != 0),
        identity: "Plantronics Voyager 8200 UC · BT600 · 047F:02EE".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "电量为 Poly 十一档估算值；仅支持只读。".into(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != "plantronics-bt600" {
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
peripheral_desk_driver_ffi::export_driver!("plantronics-bt600", handler);
