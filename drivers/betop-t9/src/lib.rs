use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "betop-t9";
const VID: u16 = 0x20bc;
const PID: u16 = 0x5073;
const REPORT_LEN: usize = 24;

fn supported(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0x00ff
        && i.usage == 0x0003
        && i.input_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&col04#")
}

fn parse(frame: &[u8]) -> Option<Option<u8>> {
    if frame.len() != REPORT_LEN || frame[0] != 0x04 {
        return None;
    }
    let raw = frame[23];
    if raw == 0x50 {
        return Some(None);
    }
    if !(0x08..=0x78).contains(&raw) || !raw.is_multiple_of(8) {
        return None;
    }
    let percent = ((u16::from(raw) * 100 + 60) / 120).min(100) as u8;
    Some(Some(percent))
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 BETOP T9 被动电量通道".into(),
            settings_error: "仅监听 T9 接收器状态流；不发送查询或配置写入。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 BETOP T9 状态接口，停止读取".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    let frame = io.read(REPORT_LEN, Duration::from_millis(300))?;
    let Some(percent) = parse(&frame) else {
        return Err("BETOP T9 暂无新的有效状态帧".into());
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if percent.is_none() {
        return Ok(State {
            key: KEY.into(),
            present: false,
            fresh: true,
            updated: now,
            identity: "BETOP T9 · 20BC:5073 · 手柄未连接".into(),
            settings_error: "接收器报告了 T9 未连接占位帧；不显示占位电量。".into(),
            ..State::default()
        });
    }
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent,
        charging: Some(false),
        updated: now,
        identity: "BETOP T9 · 20BC:5073 · 被动状态流".into(),
        settings_error: "只读电量；该上游状态流没有充电字段，不提供按键、宏或灯效设置。".into(),
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
            return Err("该插件仅被动读取 T9 电量，不发送查询或设置写入".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("betop-t9", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_offline_placeholder_and_invalid_levels() {
        let mut frame = [0; REPORT_LEN];
        frame[0] = 4;
        frame[23] = 0x58;
        assert_eq!(parse(&frame), Some(Some(73)));
        frame[23] = 0x50;
        assert_eq!(parse(&frame), Some(None));
        frame[23] = 0x59;
        assert_eq!(parse(&frame), None);
        frame[23] = 0x80;
        assert_eq!(parse(&frame), None);
    }
}
