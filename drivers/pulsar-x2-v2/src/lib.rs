use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "pulsar-x2-v2";
const VID: u16 = 0x3554;
const PID: u16 = 0xf508;
const REPORT_ID: u8 = 0x08;
const REPORT_LEN: usize = 17;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0xff02
        && i.usage == 0x0002
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&mi_01&col05#")
}

fn checksum(frame: &[u8]) -> u8 {
    0x55u8.wrapping_sub(
        frame[..REPORT_LEN - 1]
            .iter()
            .fold(0u8, |sum, value| sum.wrapping_add(*value)),
    )
}

fn request(command: u8) -> Option<[u8; REPORT_LEN]> {
    if !matches!(command, 0x03 | 0x04) {
        return None;
    }
    let mut frame = [0; REPORT_LEN];
    frame[0] = REPORT_ID;
    frame[1] = command;
    frame[16] = checksum(&frame);
    Some(frame)
}

fn parse_status(frame: &[u8]) -> Option<bool> {
    if frame.len() != REPORT_LEN
        || frame[0] != REPORT_ID
        || frame[1] != 0x03
        || frame[16] != checksum(frame)
    {
        return None;
    }
    match frame[6] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

fn parse_power(frame: &[u8]) -> Option<(u8, bool, u16)> {
    if frame.len() != REPORT_LEN
        || frame[0] != REPORT_ID
        || frame[1] != 0x04
        || frame[16] != checksum(frame)
        || frame[6] > 100
        || frame[7] > 1
    {
        return None;
    }
    Some((
        frame[6],
        frame[7] == 1,
        u16::from_be_bytes([frame[8], frame[9]]),
    ))
}

fn query(io: &mut WinTransport, command: u8) -> Result<Vec<u8>, String> {
    let request = request(command).ok_or("未知 Pulsar 查询命令")?;
    io.write(&request, Duration::from_millis(500))?;
    let until = std::time::Instant::now() + Duration::from_millis(750);
    while std::time::Instant::now() < until {
        let frame = io.read(REPORT_LEN, Duration::from_millis(150))?;
        if frame.len() == REPORT_LEN
            && frame[0] == REPORT_ID
            && frame[1] == command
            && frame[16] == checksum(&frame)
        {
            return Ok(frame);
        }
    }
    Err(format!(
        "Pulsar X2 V2 查询 {command:02X} 超时或未收到匹配回复"
    ))
}

fn read_state() -> Result<State, String> {
    let ports = hid::enumerate()?;
    let mut found = ports.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 Pulsar X2 V2 1 kHz 接收器电量接口".into(),
            settings_error:
                "仅支持 3554:F508 1 kHz 接收器；有线 F507 和 4 kHz F509 不在此插件范围。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 Pulsar X2 V2 匹配接口，停止查询".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    let online = parse_status(&query(&mut io, 0x03)?).ok_or("Pulsar STATUS 回复状态字段无效")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if !online {
        return Ok(State {
            key: KEY.into(),
            present: false,
            fresh: true,
            updated: now,
            identity: "Pulsar X2 V2 · 3554:F508 · 鼠标未连接".into(),
            settings_error: "接收器在线，但上游状态帧表明鼠标未连接。".into(),
            ..State::default()
        });
    }
    let power_reply = match query(&mut io, 0x04) {
        Ok(reply) => reply,
        Err(error) => {
            // Upstream reports that a sleeping mouse may miss the POWER reply;
            // keep the receiver present without inventing battery or offline state.
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: false,
                updated: now,
                identity: "Pulsar X2 V2 · 3554:F508 · 2.4 GHz 接收器".into(),
                error,
                settings_error: "鼠标可能处于休眠，当前没有新电量读数；不会显示为 0%。".into(),
                ..State::default()
            });
        }
    };
    let (percent, charging, voltage) =
        parse_power(&power_reply).ok_or("Pulsar POWER 回复数据无效")?;
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: Some(charging),
        updated: now,
        identity: format!("Pulsar X2 V2 · 3554:F508 · {voltage} mV"),
        settings_error: "本插件只读取电量、充电状态和上游回复的电压值；控制功能未接入。".into(),
        ..State::default()
    })
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件只读电量，不提供 DPI、回报率、按键、宏或灯光设置".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("pulsar-x2-v2", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_parses_exact_status_and_power_frames() {
        let status_request = request(0x03).unwrap();
        assert_eq!(status_request[16], 0x4a);
        let mut status = [0; REPORT_LEN];
        status[0] = REPORT_ID;
        status[1] = 0x03;
        status[6] = 1;
        status[16] = checksum(&status);
        assert_eq!(parse_status(&status), Some(true));
        let mut power = [0; REPORT_LEN];
        power[0] = REPORT_ID;
        power[1] = 0x04;
        power[6] = 20;
        power[7] = 0;
        power[8..10].copy_from_slice(&3570u16.to_be_bytes());
        power[16] = checksum(&power);
        assert_eq!(parse_power(&power), Some((20, false, 3570)));
        power[6] = 101;
        power[16] = checksum(&power);
        assert_eq!(parse_power(&power), None);
    }
}
