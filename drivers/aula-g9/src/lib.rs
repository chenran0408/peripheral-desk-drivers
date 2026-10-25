use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "aula-g9";
const VID: u16 = 0x38a6;
const PID: u16 = 0x0030;
const REPORT_LEN: usize = 64;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0xffab
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
}

fn parse(frame: &[u8]) -> Option<(Option<u8>, Option<bool>)> {
    if frame.len() != REPORT_LEN || frame[..4] != [0xab, 0x47, 0x4d, 0x86] {
        return None;
    }
    let raw = frame[6];
    let status = frame[7];
    if raw > 100 {
        return None;
    }
    if raw == 0 && status == 0 {
        return Some((None, None));
    }
    let charging = match status {
        0 => Some(false),
        2 => Some(true),
        _ => None,
    };
    Some((Some(raw), charging))
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 AULA G9 被动状态通道".into(),
            settings_error: "G9 只通过接收器推送电量状态；插件不会主动查询。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 AULA G9 状态接口，停止监听".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    let deadline = std::time::Instant::now() + Duration::from_millis(1200);
    while std::time::Instant::now() < deadline {
        let frame = io.read(REPORT_LEN, Duration::from_millis(250))?;
        if let Some((percent, charging)) = parse(&frame) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent,
                charging,
                updated: now,
                identity: "AULA G9 · 38A6:0030 · 被动状态帧".into(),
                settings_error: "只被动读取电量和已知充电状态；EQ、音量、灯光及其他设置未适配。"
                    .into(),
                ..State::default()
            });
        }
    }
    Err("AULA G9 暂无有效被动状态帧".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件仅读取 AULA G9 被动状态，不发送 HID 查询或设置写入".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("aula-g9", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_g9_status_and_keeps_zero_not_reporting_unknown() {
        let mut f = [0; REPORT_LEN];
        f[..4].copy_from_slice(&[0xab, 0x47, 0x4d, 0x86]);
        f[6] = 63;
        f[7] = 2;
        assert_eq!(parse(&f), Some((Some(63), Some(true))));
        f[6] = 0;
        f[7] = 0;
        assert_eq!(parse(&f), Some((None, None)));
        f[7] = 1;
        assert_eq!(parse(&f), Some((Some(0), None)));
        f[6] = 101;
        assert_eq!(parse(&f), None);
    }
}
