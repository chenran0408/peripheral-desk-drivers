//! Read DInput device info or the Space Station service's XInput battery status.
mod fcs;

use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use windows_sys::Win32::Devices::{
    DeviceAndDriverInstallation::{
        CM_Get_Device_IDW, CM_Get_Device_Interface_PropertyW, CM_Get_Parent, CM_Locate_DevNodeW,
        CR_SUCCESS, MAX_DEVICE_ID_LEN,
    },
    Properties::DEVPKEY_Device_InstanceId,
};

const MODEL: &str = "flydigi-vader-4-pro";
const INFO_COMMAND: u8 = 0xec;
const INPUT_LENGTH: usize = 32;

#[derive(Debug, PartialEq, Eq)]
struct Reading {
    raw: Option<u8>,
    percent: Option<u8>,
    charging: Option<bool>,
    wired: Option<bool>,
}

fn parse_info(report: &[u8]) -> Option<Reading> {
    if report.len() != INPUT_LENGTH
        || report[0] != 0x04
        || report[15] != INFO_COMMAND
        || !matches!(report[3], 85 | 91 | 105)
    {
        return None;
    }
    let raw = report[11];
    let (percent, charging) = match (raw >> 4, raw & 0x0f) {
        (0, level @ 1..=5) => (Some(20 * level), Some(false)),
        (1, level @ 0..=5) => (Some(20 * level), Some(true)),
        (1, _) => (None, Some(true)),
        (2, _) => (Some(100), Some(false)),
        // A freshly connected dongle may answer before it knows the level.
        _ => (None, None),
    };
    let wired = match report[13] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Some(Reading {
        raw: Some(raw),
        percent,
        charging,
        wired,
    })
}

fn endpoint(i: &hid::Interface) -> bool {
    i.vid == 0x04b4
        && matches!(i.pid, 0x2412 | 0x2414)
        && i.path.to_ascii_lowercase().contains("&mi_02")
        && usize::from(i.input_len) == INPUT_LENGTH
        && (12..=64).contains(&i.output_len)
}

fn xinput_endpoint(i: &hid::Interface) -> bool {
    if (i.vid, i.pid) != (0x045e, 0x028e) || !i.path.to_ascii_lowercase().contains("&ig_00") {
        return false;
    }
    // XUSB exposes the same HID VID/PID for genuine Xbox pads and many clones.
    // The USB parent of the Vader 4 Pro contains the actual device name.
    let mut instance = [0u16; MAX_DEVICE_ID_LEN as usize];
    let mut size = std::mem::size_of_val(&instance) as u32;
    let (mut kind, mut node) = (0, 0);
    unsafe {
        if CM_Get_Device_Interface_PropertyW(
            hid::wide(&i.path).as_ptr(),
            &DEVPKEY_Device_InstanceId,
            &mut kind,
            instance.as_mut_ptr().cast(),
            &mut size,
            0,
        ) != CR_SUCCESS
            || CM_Locate_DevNodeW(&mut node, instance.as_ptr(), 0) != CR_SUCCESS
        {
            return false;
        }
        for _ in 0..3 {
            let mut parent = 0;
            if CM_Get_Parent(&mut parent, node, 0) != CR_SUCCESS
                || CM_Get_Device_IDW(parent, instance.as_mut_ptr(), instance.len() as u32, 0)
                    != CR_SUCCESS
            {
                return false;
            }
            let len = instance
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(instance.len());
            if String::from_utf16_lossy(&instance[..len])
                .eq_ignore_ascii_case(r"USB\VID_045E&PID_028E\FLYDIGI_VADER4")
            {
                return true;
            }
            node = parent;
        }
    }
    false
}

fn query(i: &hid::Interface) -> Result<Reading, String> {
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    // Windows WriteFile takes a buffer padded to OutputReportByteLength.
    let mut command = vec![0; usize::from(i.output_len)];
    command[..2].copy_from_slice(&[0x05, INFO_COMMAND]);
    io.write(&command, Duration::from_millis(250))?;
    let deadline = Instant::now() + Duration::from_millis(550);
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let report = io.read(INPUT_LENGTH, left)?;
        if report.is_empty() {
            break;
        }
        if let Some(reading) = parse_info(&report) {
            return Ok(reading);
        }
    }
    Err("未收到黑武士 4 Pro 的电量回包。请唤醒手柄并确认 PC 档已切到蓝灯 DInput。".into())
}

fn state(i: &hid::Interface, instance_id: String, reading: Reading, connection: &str) -> State {
    State {
        key: MODEL.into(),
        instance_id,
        present: true,
        fresh: true,
        percent: reading.percent,
        battery_raw: reading.raw,
        connection_path: Some(connection.into()),
        charging: reading.charging,
        wired: reading.wired == Some(true),
        identity: format!("Flydigi Vader 4 Pro · {:04X}:{:04X}", i.vid, i.pid),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    }
}

fn read_state(request: &Value) -> Result<State, String> {
    let target = hid::request_instance(request)?;
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces
        .iter()
        .filter(|i| endpoint(i))
        .map(|i| (i, i.instance_id()))
        .filter(|(_, id)| target.is_none_or(|target| target == id))
        .collect();
    let mut xbox = interfaces
        .iter()
        .filter(|i| xinput_endpoint(i))
        .map(|i| (i, i.instance_id()))
        .collect::<Vec<_>>();
    xbox.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    xbox.dedup_by(|a, b| a.1 == b.1);
    if let Some(index) = xbox
        .iter()
        .position(|(_, id)| target.is_none_or(|target| target == id))
    {
        if xbox.len() != 1 || (!candidates.is_empty() && target.is_none()) {
            return Err("有多台黑武士 4 Pro，空间站服务无法将电量对应到一台白灯手柄。".into());
        }
        let (interface, instance_id) = xbox.swap_remove(index);
        // Without the service's VID/PID, a second Vader in DInput could be
        // mistaken for this XInput target.
        let require_xinput_id = interfaces.iter().any(endpoint);
        let reading = fcs::query_vader4(require_xinput_id)?;
        let connection = match reading.wired {
            Some(true) => "XInput · USB 有线 · 飞智空间站",
            Some(false) => "XInput · 2.4 GHz 接收器 · 飞智空间站",
            None => "XInput · 飞智空间站",
        };
        return Ok(state(interface, instance_id, reading, connection));
    }
    let Some((_, instance_id)) = candidates.first() else {
        return Ok(State {
            key: MODEL.into(),
            error: "未检测到黑武士 4 Pro。".into(),
            ..State::default()
        });
    };
    if candidates.iter().any(|(_, id)| id != instance_id) {
        return Err("检测到多个飞智接口，请选择要查看的手柄。".into());
    }
    let mut error = String::new();
    for (i, instance_id) in candidates {
        match query(i) {
            Ok(reading) => {
                let connection = match reading.wired {
                    Some(true) => "DInput · USB 有线",
                    Some(false) => "DInput · 2.4 GHz 接收器",
                    None => "DInput",
                };
                return Ok(state(i, instance_id, reading, connection));
            }
            Err(failure) => error = failure,
        }
    }
    Err(error)
}

fn handle(key: &str, input: &str) -> String {
    let result = (|| {
        let request: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
        if key != MODEL || request["op"].as_str() != Some("read-state") {
            return Err("此插件只读取黑武士 4 Pro 的电量。".into());
        }
        read_state(&request)
    })();
    match result {
        Ok(state) => json!({"ok": true, "value": state, "error": ""}),
        Err(error) => json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("flydigi-vader-4-pro", handle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_vader_4_pro_info_reports_supply_a_battery_tier() {
        let mut report = [0; 32];
        report[0] = 0x04;
        report[3] = 85;
        report[11] = 2;
        report[15] = INFO_COMMAND;
        let reading = parse_info(&report).unwrap();
        assert_eq!(
            (reading.percent, reading.charging, reading.wired),
            (Some(40), Some(false), Some(false))
        );
        report[11] = 0;
        assert_eq!(parse_info(&report).unwrap().percent, None);
        report[11] = 0x13;
        report[13] = 1;
        let reading = parse_info(&report).unwrap();
        assert_eq!(
            (reading.percent, reading.charging, reading.wired),
            (Some(60), Some(true), Some(true))
        );
        report[11] = 0x20;
        let reading = parse_info(&report).unwrap();
        assert_eq!(
            (reading.percent, reading.charging),
            (Some(100), Some(false))
        );
        report[3] = 80; // Vader 3 Pro shares the family protocol.
        assert!(parse_info(&report).is_none());
        report[3] = 85;
        report[15] = 0;
        assert!(parse_info(&report).is_none());
    }
}
