use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "betop-kunpeng20";
const REPORT_LEN: usize = 64;

fn supported(i: &hid::Interface) -> bool {
    i.vid == 0x20bc
        && matches!(i.pid, 0x5158 | 0x5159)
        && i.usage_page == 0x00ff
        && i.usage == 0x0003
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.product.to_ascii_lowercase().contains("kp20")
}

fn parse(frame: &[u8]) -> Option<(u8, Option<bool>)> {
    if frame.len() != REPORT_LEN || frame[0] != 0x02 || frame[1] != 0x15 {
        return None;
    }
    let percent = frame[2];
    if !(1..=100).contains(&percent) {
        return None;
    }
    let charging = match frame[4] & 0x0f {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Some((percent, charging))
}

fn query() -> [u8; REPORT_LEN] {
    let mut frame = [0; REPORT_LEN];
    frame[0] = 0x02;
    frame[1] = 0x15;
    frame
}

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut found = interfaces.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 BETOP Kunpeng 20 电量接口".into(),
            settings_error: "只支持 VID_20BC 的 5158 有线或 5159 接收器；蓝牙和 5191 变体未接入。"
                .into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 BETOP Kunpeng 20 电量接口，停止查询".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    let wireless = interface.pid == 0x5159;
    let response = if wireless {
        // The documented 5159 dongle broadcasts once per second; remain passive.
        io.read(REPORT_LEN, Duration::from_millis(1200))?
    } else {
        Vec::new()
    };
    let frame = if parse(&response).is_some() {
        response
    } else {
        io.flush()?;
        io.write(&query(), Duration::from_millis(500))?;
        let deadline = std::time::Instant::now() + Duration::from_millis(1000);
        let mut matched = Vec::new();
        while std::time::Instant::now() < deadline {
            let reply = io.read(REPORT_LEN, Duration::from_millis(180))?;
            if parse(&reply).is_some() {
                matched = reply;
                break;
            }
        }
        matched
    };
    let Some((percent, charging)) = parse(&frame) else {
        return Err("BETOP Kunpeng 20 未收到电量字段有效的状态回复".into());
    };
    Ok(State {
        key: KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging,
        wired: interface.pid == 0x5158,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: format!(
            "BETOP Kunpeng 20 · 20BC:{:04X} · {}",
            interface.pid,
            if wireless {
                "2.4 GHz 接收器"
            } else {
                "USB 有线"
            }
        ),
        settings_error: "只读取电量和已知充电状态；手柄按键、宏、灯光与配置控制未接入。".into(),
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
            return Err("该插件只读手柄电量，不提供配置写入".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("betop-kunpeng20", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exact_status_and_rejects_echoes_and_unknown_flags() {
        let mut f = [0; REPORT_LEN];
        f[0] = 2;
        f[1] = 0x15;
        f[2] = 39;
        f[4] = 0x50;
        assert_eq!(parse(&f), Some((39, Some(false))));
        f[4] = 0x51;
        assert_eq!(parse(&f), Some((39, Some(true))));
        f[4] = 0x52;
        assert_eq!(parse(&f), Some((39, None)));
        f[2] = 0;
        assert_eq!(parse(&f), None);
        f[2] = 39;
        f[1] = 0x14;
        assert_eq!(parse(&f), None);
    }
}
