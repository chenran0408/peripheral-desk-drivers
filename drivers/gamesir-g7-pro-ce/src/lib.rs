use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "gamesir-g7-pro-ce";
const VID: u16 = 0x3537;
const PAGE: u16 = 0xfff0;
const USAGE: u16 = 0x0040;
const REPORT_LEN: usize = 64;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
}

fn heartbeat() -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[0] = 0x0f;
    frame[1] = 0xf2;
    frame[2] = 0;
    frame
}

fn parse(frame: &[u8]) -> Option<(u8, bool)> {
    if frame.len() != REPORT_LEN || frame[0] != 0x12 || frame[36] > 100 {
        return None;
    }
    Some((frame[36], frame[35] == 1))
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 GameSir G7 Pro CE 状态接口".into(),
            settings_error: "需启用插件后发送一次状态心跳；仅支持 VID 3537 与 FFF0:0040 集合。"
                .into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 GameSir G7 Pro CE 状态接口，停止心跳".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    io.write(&heartbeat(), Duration::from_millis(500))?;
    let deadline = std::time::Instant::now() + Duration::from_millis(1800);
    while std::time::Instant::now() < deadline {
        let frame = io.read(REPORT_LEN, Duration::from_millis(180))?;
        if let Some((percent, charging)) = parse(&frame) {
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
                identity: format!(
                    "GameSir G7 Pro CE · 3537:{:04X} · 2.4 GHz 接收器",
                    interface.pid
                ),
                settings_error: "每次刷新会发送一条状态心跳；本插件不更改控制器配置。".into(),
                ..State::default()
            });
        }
    }
    Err("GameSir G7 Pro CE 心跳后未收到匹配的 0x12 状态帧".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件仅通过状态心跳读取电量，不写手柄设置".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("gamesir-g7-pro-ce", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_documented_heartbeat_and_parses_status_fields() {
        let beat = heartbeat();
        assert_eq!(&beat[..3], &[0x0f, 0xf2, 0]);
        assert!(beat[3..].iter().all(|byte| *byte == 0));
        let mut report = [0; REPORT_LEN];
        report[0] = 0x12;
        report[35] = 1;
        report[36] = 100;
        assert_eq!(parse(&report), Some((100, true)));
        report[0] = 0x11;
        assert_eq!(parse(&report), None);
        report[0] = 0x12;
        report[36] = 101;
        assert_eq!(parse(&report), None);
    }
}
