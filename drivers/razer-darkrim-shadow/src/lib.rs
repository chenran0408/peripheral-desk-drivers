use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::{
    mem::size_of,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
};

const KEY: &str = "razer-darkrim-shadow";
const VID: u16 = 0x3654;
const PID: u16 = 0x6901;
const REPORT_LEN: usize = 65;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0xff02
        && i.usage == 0x0002
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&mi_03#")
}

fn official_client_running() -> Result<bool, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err("无法检查 DARKRIM.exe 是否运行；为避免争用设备，插件暂不读取".into());
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snapshot, &mut entry);
        if ok == 0 {
            CloseHandle(snapshot);
            return Err("无法枚举进程状态；为避免争用设备，插件暂不读取".into());
        }
        let mut found = false;
        while ok != 0 {
            let end = entry
                .szExeFile
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
            if name.eq_ignore_ascii_case("DARKRIM.exe") {
                found = true;
                break;
            }
            ok = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        Ok(found)
    }
}

fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0xffffu16;
    for byte in data {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn request() -> [u8; REPORT_LEN] {
    let logical = [
        0xfe, 0x00, 0x10, 0x00, 0x02, 0x00, 0x00, 0x03, 0x00, 0x09, 0x01, 0x00,
    ];
    let crc = crc16(&logical);
    let mut report = [0; REPORT_LEN];
    report[1..1 + logical.len()].copy_from_slice(&logical);
    report[1 + logical.len()] = (crc >> 8) as u8;
    report[2 + logical.len()] = crc as u8;
    report
}

fn parse_battery(report: &[u8]) -> Option<(u8, bool)> {
    if report.len() != REPORT_LEN || report[0] != 0 {
        return None;
    }
    let body = &report[1..];
    if body[0] != 0xfe
        || body[2] != 0x11
        || body[3] != 0
        || body[4] != 0x03
        || body[5] != 0
        || body[6] != 0
        || body[8] != 0
        || body[9] != 0x01
        || body[10] != 0x01
    {
        return None;
    }
    let payload_len = body[7] as usize;
    if payload_len != 3 {
        return None;
    }
    let crc_offset = 9 + payload_len;
    let expected = crc16(&body[..crc_offset]);
    let actual = u16::from_be_bytes([body[crc_offset], body[crc_offset + 1]]);
    if expected != actual {
        return None;
    }
    let raw = body[11];
    let percent = raw & 0x7f;
    if !(1..=100).contains(&percent) {
        return None;
    }
    Some((percent, raw & 0x80 != 0))
}

fn read_state() -> Result<State, String> {
    match official_client_running() {
        Ok(true) => {
            return Ok(State {
                key: KEY.into(),
                present: false,
                error: "DARKRIM 官方程序正在运行，插件已让位且未打开设备接口".into(),
                settings_error:
                    "关闭官方 DARKRIM 客户端后，下次刷新即可读取；避免两个程序争用接收器。".into(),
                ..State::default()
            });
        }
        Ok(false) => {}
        Err(error) => return Err(error),
    }
    let ports = hid::enumerate()?;
    let mut found = ports.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 DARKRIM SHADOW MI_03 电量接口".into(),
            settings_error: "仅支持 3654:6901 / FF02:0002 / MI_03 / 65 字节查询通道。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 DARKRIM SHADOW 查询接口，停止访问".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    io.write(&request(), Duration::from_millis(500))?;
    let deadline = std::time::Instant::now() + Duration::from_millis(1200);
    while std::time::Instant::now() < deadline {
        let reply = io.read(REPORT_LEN, Duration::from_millis(150))?;
        if let Some((percent, charging)) = parse_battery(&reply) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                identity: "DARKRIM SHADOW · 3654:6901 · MI_03".into(),
                settings_error: "仅主动读取电量和充电状态；EQ、音量、灯效及设备设置未适配。".into(),
                ..State::default()
            });
        }
    }
    Err("DARKRIM SHADOW 未收到 CRC 正确的匹配电量应答".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件仅在允许主动读取时查询电量，不提供音频设置写入".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("razer-darkrim-shadow", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_documented_query_and_parses_only_crc_valid_battery_tlv() {
        let query = request();
        assert_eq!(
            &query[1..15],
            &[0xfe, 0, 0x10, 0, 2, 0, 0, 3, 0, 9, 1, 0, 0xe2, 0x3c]
        );
        let mut body = [0xfe, 0, 0x11, 0, 3, 0, 0, 3, 0, 1, 1, 0xda, 0, 0];
        let crc = crc16(&body[..12]);
        body[12] = (crc >> 8) as u8;
        body[13] = crc as u8;
        let mut report = [0; REPORT_LEN];
        report[1..15].copy_from_slice(&body);
        assert_eq!(parse_battery(&report), Some((90, true)));
        report[14] ^= 1;
        assert_eq!(parse_battery(&report), None);
    }
}
