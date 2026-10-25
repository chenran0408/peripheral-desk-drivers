use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "eweadn-x87";
const VID: u16 = 0x0c45;
const PID: u16 = 0xfefe;
const REPORT_LEN: usize = 33;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0xff60
        && i.usage == 0x0061
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&mi_03&")
}

fn checksum(frame: &[u8]) -> u8 {
    frame[..REPORT_LEN - 1]
        .iter()
        .fold(0u8, |sum, value| sum.wrapping_add(*value))
}

fn request() -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[1] = 0x20;
    frame[2] = 0x01;
    frame[REPORT_LEN - 1] = checksum(&frame);
    frame
}

fn parse(frame: &[u8]) -> Option<u8> {
    if frame.len() != REPORT_LEN
        || frame[1] != 0x20
        || frame[2] != 0x01
        || frame[4] == 0
        || frame[4] > 100
        || frame[REPORT_LEN - 1] != checksum(frame)
    {
        return None;
    }
    Some(frame[4])
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 EWEADN X87 接收器 MI_03 电量通道".into(),
            settings_error: "仅适配 0C45:FEFE 接收器电量；改键、宏和灯光未接入。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 EWEADN X87 电量接口，停止查询".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    io.write(&request(), Duration::from_millis(500))?;
    let until = std::time::Instant::now() + Duration::from_millis(900);
    while std::time::Instant::now() < until {
        let reply = io.read(REPORT_LEN, Duration::from_millis(150))?;
        if let Some(percent) = parse(&reply) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                identity: "EWEADN X87 · 0C45:FEFE · MI_03".into(),
                settings_error: "只读取接收器电量；上游没有确认充电字段或键盘控制写入。".into(),
                ..State::default()
            });
        }
    }
    Err("EWEADN X87 接收器未返回匹配且校验正确的电量帧".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件只读电量，不提供按键、宏或灯光写入".to_string());
        }
        read_state().map_err(|error| error.to_string())
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("eweadn-x87", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_exact_x87_response_with_valid_sum() {
        let mut frame = [0; REPORT_LEN];
        frame[1] = 0x20;
        frame[2] = 1;
        frame[4] = 73;
        frame[32] = checksum(&frame);
        assert_eq!(parse(&frame), Some(73));
        frame[4] = 101;
        frame[32] = checksum(&frame);
        assert_eq!(parse(&frame), None);
        frame[4] = 73;
        frame[32] ^= 1;
        assert_eq!(parse(&frame), None);
    }
}
