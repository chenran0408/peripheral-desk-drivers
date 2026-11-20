//! MCHOSE universal 8K receiver: a model is identified by the paired device's 0x0900 response.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const RECEIVER: u16 = 0x1018;
const VID: u16 = 0x3837;
const LEN: usize = 64;
const DEVICE_INFO: u16 = 0x0900;
const PERFORMANCE: u16 = 0x0002;
const DPI: u16 = 0x0003;
const STAGE_EVENT: u16 = 0x0600;

fn query(command: u16, args: &[u8]) -> [u8; LEN] {
    let mut frame = [0; LEN];
    let [lo, hi] = command.to_le_bytes();
    frame[..8].copy_from_slice(&[0x4d, 1, 1, args.len() as u8, lo, hi, 0, 0]);
    let end = 8 + args.len();
    frame[8..end].copy_from_slice(args);
    frame[end] = frame[2..end].iter().fold(0, |sum, b| sum ^ b);
    frame
}

#[derive(Debug, Eq, PartialEq)]
struct Info {
    vid: u16,
    pid: u16,
    percent: u8,
    charge: u8,
    connection: u8,
}

fn payload(frame: &[u8], command: u16) -> Option<&[u8]> {
    // Windows can retain a leading report slot before the 0x4D frame.
    let offset = frame
        .windows(2)
        .take(2)
        .position(|header| header == [0x4d, 1])?;
    let message = &frame[offset..];
    if message.len() < 8 || message[4..6] != command.to_le_bytes() {
        return None;
    }
    // The published receiver capture does not use the outgoing XOR convention.
    message.get(8..8 + usize::from(message[3]))
}

fn info(frame: &[u8]) -> Option<Info> {
    let payload = payload(frame, DEVICE_INFO)?;
    // A zero-length echo and a sleeping receiver's one-byte FF have no identity.
    if payload.len() < 14 || !(1..=100).contains(&payload[12]) {
        return None;
    }
    Some(Info {
        vid: u16::from_le_bytes([payload[0], payload[1]]),
        pid: u16::from_le_bytes([payload[2], payload[3]]),
        percent: payload[12],
        charge: payload[11],
        connection: payload[10],
    })
}

fn stage_event(frame: &[u8]) -> Option<u8> {
    let data = payload(frame, STAGE_EVENT)?;
    let stage = *data.get(2)?;
    (data[0] & 1 != 0 && stage < 6).then_some(stage)
}

fn rate_indices(frame: &[u8]) -> Option<[u8; 2]> {
    let [_, usb, wireless, ..] = payload(frame, PERFORMANCE)? else {
        return None;
    };
    // Mirror the vendor's decoder, including wire values 1 and 2 both mapping
    // to table index 1. The lower nibble of the first byte is a different field.
    let index = |value: u8| match value >> 4 {
        0 => 0,
        1 => 1,
        wire => wire - 1,
    };
    Some([index(*usb), index(*wireless)])
}

fn configured_rate(model: u16, receiver: u16, indices: [u8; 2]) -> Result<u16, String> {
    if indices[0] != indices[1] {
        return Err("两个回报率配置索引不一致，无法确定当前配置".into());
    }
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../catalog/upstream/vigil-watt/mchose-catalog.json"
    ))
    .expect("checked-in MCHOSE catalog");
    let table = catalog["rateProfiles"][format!("0x{model:04x}")][format!("0x{receiver:04x}")]
        .as_str()
        .ok_or("官方目录没有这组鼠标与接收器的回报率表")?;
    catalog["rateTables"][table]
        .get(usize::from(indices[0]))
        .and_then(Value::as_u64)
        .and_then(|hz| u16::try_from(hz).ok())
        .ok_or_else(|| "回报率索引超出该型号的配置表".into())
}

struct DpiConfig {
    stages: Vec<u16>,
    separate_axes: bool,
}

fn dpi_config(frame: &[u8], direction: u8) -> Option<DpiConfig> {
    let data = payload(frame, DPI)?;
    // The upstream probe requests profile zero. This profile has no proven
    // relationship to the active profile, so its entries are settings only.
    let slots = data.get(5..17)?;
    let count = usize::from(data[2]);
    if data[0] != 0
        || data[1] != direction
        || !(1..=6).contains(&count)
        || usize::from(data[3]) >= count
        || data[4] > 1
    {
        return None;
    }
    let stages: Vec<_> = slots
        .as_chunks::<2>()
        .0
        .iter()
        .take(count)
        .map(|&slot| u16::from_le_bytes(slot))
        .collect();
    if stages.contains(&0) {
        return None;
    }
    Some(DpiConfig {
        stages,
        separate_axes: data[4] == 1,
    })
}

fn matches(interface: &hid::Interface) -> bool {
    interface.vid == VID
        && interface.pid == RECEIVER
        && interface.usage_page == 0xff01
        && interface.usage == 1
        && interface.input_len == LEN as u16
        && interface.output_len == LEN as u16
}

fn read_reply<T>(
    transport: &mut WinTransport,
    command: u16,
    args: &[u8],
    stage: &mut Option<u8>,
    parse: impl Fn(&[u8]) -> Option<T>,
) -> Result<T, String> {
    transport.write(&query(command, args), Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(1200);
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        let frame = transport.read(LEN, remaining)?;
        if frame.is_empty() {
            break;
        }
        if let Some(response) = parse(&frame) {
            return Ok(response);
        }
        if let Some(observed) = stage_event(&frame) {
            *stage = Some(observed);
        }
    }
    Err(format!(
        "接收器未返回有效的 0x{command:04X} 读数，请确认鼠标已唤醒"
    ))
}

fn state(key: &str, expected_pid: u16, report: Info) -> Result<State, String> {
    if (report.vid, report.pid) != (VID, expected_pid) {
        return Err(format!(
            "接收器当前配对的设备自报 {:04X}:{:04X}，所选型号需要 3837:{expected_pid:04X}",
            report.vid, report.pid
        ));
    }
    let link = match report.connection {
        1 => "设备自报有线",
        2 => "设备自报 2.4 GHz",
        _ => "设备连接方式未知",
    };
    Ok(State {
        key: key.into(),
        present: true,
        fresh: true,
        percent: Some(report.percent),
        battery_raw: Some(report.percent),
        charging: match report.charge {
            0 | 2 => Some(false),
            1 => Some(true),
            _ => None,
        },
        wired: report.connection == 1,
        connection_path: Some(format!("3837:1018 · FF01:0001 · {link}")),
        identity: format!("0x0900 配对身份 3837:{expected_pid:04X}"),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    })
}

fn read_state(
    path: &str,
    key: &str,
    expected_pid: u16,
    read_pointer: bool,
) -> Result<Value, String> {
    let mut transport = WinTransport::open(path)?;
    transport.flush()?;
    let mut stage = None;
    let report = read_reply(&mut transport, DEVICE_INFO, &[], &mut stage, info)?;
    let mut state = state(key, expected_pid, report)?;
    let mut indices = None;
    let mut dpi_y = None;
    if read_pointer {
        // Device details request these explicitly; background battery polls
        // never read performance settings. Independent failures preserve power.
        let mut errors = Vec::new();
        let rate = read_reply(&mut transport, PERFORMANCE, &[], &mut stage, rate_indices).and_then(
            |pair| {
                indices = Some(pair);
                configured_rate(expected_pid, RECEIVER, pair)
            },
        );
        match rate {
            Ok(hz) => state.rate = Some(hz),
            Err(reason) => errors.push(format!("配置回报率：{reason}")),
        }
        let mut axis = |direction| {
            read_reply(&mut transport, DPI, &[0, direction], &mut stage, |frame| {
                dpi_config(frame, direction)
            })
        };
        match axis(0) {
            Ok(x) => {
                state.dpi_stages = x.stages;
                if x.separate_axes {
                    match axis(1) {
                        Ok(y) => dpi_y = Some(y.stages),
                        Err(reason) => errors.push(format!("Y 轴 DPI 配置：{reason}")),
                    }
                }
            }
            Err(reason) => errors.push(format!("X 轴 DPI 配置：{reason}")),
        }
        state.settings_error = errors.join("。");
    }
    state.active_stage = stage;
    let mut value = json!(state);
    if let Some(pair) = indices {
        value["rate_indices"] = json!(pair);
    }
    if let Some(values) = dpi_y {
        value["dpi_stages_y"] = json!(values);
    }
    Ok(value)
}

pub fn request(key: &str, input: &str, expected_pid: u16) -> String {
    let result = (|| {
        let op: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if op.get("op").and_then(Value::as_str) != Some("read-state") {
            return Err("此插件支持读取设备状态".into());
        }
        let read_pointer = op.get("read_pointer").and_then(Value::as_bool) == Some(true);
        let mut error = "未找到 3837:1018 的 8K 接收器命令接口".to_string();
        for interface in hid::enumerate()?.iter().filter(|i| matches(i)) {
            match read_state(&interface.path, key, expected_pid, read_pointer) {
                Ok(result) => return Ok(result),
                Err(message) => error = message,
            }
        }
        Err(error)
    })();
    match result {
        Ok(value) => json!({"ok": true, "value": value, "error": ""}),
        Err(error) => json!({"ok": false, "value": null, "error": error}),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    // Upstream physical capture, docs/hardware/brands/mchose.md section 3.3.
    const CAPTURE: &[u8] = &[
        0x4d, 1, 1, 16, 0, 9, 0, 0, 0x37, 0x38, 0x26, 0x40, 4, 0, 0, 0, 0, 0x10, 2, 0, 0x64, 0, 8,
        0xe4, 0xef,
    ];

    #[test]
    fn capture_and_paired_identity_are_independent_of_the_receiver() {
        let captured = info(CAPTURE).unwrap();
        assert_eq!(
            (
                captured.pid,
                captured.percent,
                captured.charge,
                captured.connection
            ),
            (0x4026, 100, 0, 2)
        );
        assert!(state("mchose-k7-v2-ultra-plus", 0x4028, captured).is_err());
        let mut reply = [vec![0], CAPTURE.to_vec()].concat();
        reply[11] = 0x28;
        reply[20] = 1;
        reply[21] = 71;
        let state = state("mchose-k7-v2-ultra-plus", 0x4028, info(&reply).unwrap()).unwrap();
        assert_eq!(
            (state.percent, state.charging, state.wired),
            (Some(71), Some(true), false)
        );
    }

    #[test]
    fn echo_short_wrong_command_and_invalid_battery_are_not_readings() {
        assert!(info(&query(DEVICE_INFO, &[])).is_none());
        assert!(info(&[0x4d, 1, 1, 1, 0, 9, 0, 0, 0xff, 0]).is_none());
        for (index, value) in [(3, 32), (5, 6), (20, 0), (20, 101)] {
            let mut frame = CAPTURE.to_vec();
            frame[index] = value;
            assert!(info(&frame).is_none());
        }
    }

    #[test]
    fn report_slot_low_battery_and_unknown_status() {
        let mut report = [vec![0x4d], CAPTURE.to_vec()].concat();
        report[19] = 0;
        report[20] = 9;
        report[21] = 1;
        let reading = state("sample", 0x4026, info(&report).unwrap()).unwrap();
        assert_eq!((reading.percent, reading.charging), (Some(1), None));
        assert!(reading.connection_path.unwrap().contains("未知"));
    }

    #[test]
    fn query_uses_only_device_info_and_other_operations_are_rejected() {
        let device_info = query(DEVICE_INFO, &[]);
        assert_eq!(&device_info[..9], &[0x4d, 1, 1, 0, 0, 9, 0, 0, 8]);
        assert!(device_info[9..].iter().all(|&byte| byte == 0));
        assert_eq!(
            &query(PERFORMANCE, &[])[..9],
            &[0x4d, 1, 1, 0, 2, 0, 0, 0, 3]
        );
        assert_eq!(
            &query(DPI, &[0, 1])[..11],
            &[0x4d, 1, 1, 2, 3, 0, 0, 0, 0, 1, 1]
        );
        let response: Value =
            serde_json::from_str(&request("sample", r#"{"op":"apply"}"#, 0x4028)).unwrap();
        assert_eq!(response["ok"], false);
    }

    #[test]
    fn rate_requires_matching_indices_and_a_table_for_model_and_receiver() {
        let mut frame = [0x4d, 1, 1, 3, 2, 0, 0, 0, 0, 0x63, 0x60];
        let pair = rate_indices(&frame).unwrap();
        assert_eq!(configured_rate(0x4028, RECEIVER, pair).unwrap(), 8000);
        assert!(configured_rate(0x4040, RECEIVER, pair).is_err());
        frame[9] = 0x10;
        frame[10] = 0x20;
        assert_eq!(
            configured_rate(0x4028, RECEIVER, rate_indices(&frame).unwrap()).unwrap(),
            500
        );
        assert!(configured_rate(0x4028, RECEIVER, [2, 3]).is_err());
        assert!(configured_rate(0x4028, RECEIVER, [7, 7]).is_err());
    }

    #[test]
    fn only_a_marked_stage_event_gives_an_active_stage() {
        let mut event = [0x4d, 1, 1, 3, 0, 6, 0, 0, 1, 0, 5];
        assert_eq!(stage_event(&event), Some(5));
        event[10] = 6;
        assert_eq!(stage_event(&event), None);
        event[10] = 0;
        event[8] = 2;
        assert_eq!(stage_event(&event), None);
        assert_eq!(stage_event(CAPTURE), None);
    }

    #[test]
    fn dpi_axes_preserve_profile_zero_values_and_declared_count() {
        let mut frame = [
            0x4d, 1, 1, 17, 3, 0, 0, 0, 0, 0, 2, 1, 1, 0x90, 1, 0x20, 3, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let x = dpi_config(&frame, 0).unwrap();
        assert!(x.separate_axes);
        assert_eq!(x.stages, [400, 800]);
        assert!(dpi_config(&frame, 1).is_none());
        frame[9] = 1;
        assert_eq!(dpi_config(&frame, 1).unwrap().stages, [400, 800]);
        frame[8] = 1;
        assert!(dpi_config(&frame, 1).is_none());
        frame[8] = 0;
        for (offset, value) in [(3, 16), (10, 0), (10, 7), (11, 2), (12, 2)] {
            let mut invalid = frame;
            invalid[offset] = value;
            assert!(dpi_config(&invalid, 1).is_none());
        }
    }
}
