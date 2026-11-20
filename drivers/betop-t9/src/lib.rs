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
        && matches!((i.usage_page, i.usage), (0xff00, 1..=3) | (1..=3, 0xff00))
        && i.input_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&col04#")
}

fn parse(frame: &[u8]) -> Option<(bool, Option<u8>)> {
    if frame.len() != REPORT_LEN || frame[0] != 0x04 {
        return None;
    }
    let connected = !matches!(frame[23], 0x50 | 0x10);
    let level = usize::from(frame[1] & 7);
    let percent = if connected {
        [0, 50, 80, 100, 100].get(level).copied()
    } else {
        None
    };
    Some((connected, percent))
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
    let Some((connected, percent)) = parse(&frame) else {
        return Err("BETOP T9 暂无新的有效状态帧".into());
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if !connected {
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
        battery_raw: Some(frame[1]),
        connection_path: Some("2.4 GHz · 20BC:5073 · Col04 被动上报".into()),
        updated: now,
        identity: "BETOP T9 · 20BC:5073 · 被动状态流".into(),
        settings_error: "电量按手柄上报的档位显示，充电状态未知。".into(),
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
    fn reports_the_documented_levels_and_link_flags_independently() {
        let mut frame = [0; REPORT_LEN];
        frame[0] = 4;
        frame[23] = 0x58;
        for (flags, percent) in [
            (0, Some(0)),
            (1, Some(50)),
            (2, Some(80)),
            (3, Some(100)),
            (0x0c, Some(100)),
            (5, None),
        ] {
            frame[1] = flags;
            assert_eq!(parse(&frame), Some((true, percent)));
        }
        frame[23] = 0x50;
        assert_eq!(parse(&frame), Some((false, None)));
        frame[23] = 0x10;
        frame[1] = 0;
        assert_eq!(parse(&frame), Some((false, None)));
        frame[0] = 5;
        assert_eq!(parse(&frame), None);
    }
}
