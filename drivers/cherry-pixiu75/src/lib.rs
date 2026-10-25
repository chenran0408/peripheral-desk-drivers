use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KEY: &str = "cherry-pixiu75";
const REPORT: u8 = 0x04;
const REPORT_LEN: usize = 64;
const BLOCK: u8 = 0x1a;

fn supported(i: &hid::Interface) -> bool {
    matches!((i.vid, i.pid), (0x046a, 0x01e4) | (0x046a, 0x01e2))
        && i.usage_page == 0xff1c
        && i.usage == 0x0092
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
        && i.path.to_ascii_lowercase().contains("&col04#")
}

fn request() -> [u8; REPORT_LEN] {
    let mut f = [0; REPORT_LEN];
    f[0] = REPORT;
    f[3] = BLOCK;
    f[4] = 4;
    f
}

fn parse_reply(f: &[u8]) -> Option<(u8, bool)> {
    if f.len() != REPORT_LEN
        || f[0] != REPORT
        || f[1] != 0
        || f[2] != 0
        || f[3] != BLOCK
        || f[4] != 4
        || f[5] != 0
        || f[6] != 0
        || f[7] != 0
        || f[8] > 100
        || f[9] & 0xfe != 0
    {
        return None;
    }
    Some((f[8], f[9] & 1 != 0))
}

fn read_state() -> Result<State, String> {
    let ports = hid::enumerate()?;
    let mut found = ports.iter().filter(|i| supported(i));
    let Some(interface) = found.next() else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 CHERRY PIXIU75 块电量接口".into(),
            settings_error: "仅读取 PIXIU75 电量和充电状态；键位、宏及灯效未接入。".into(),
            ..State::default()
        });
    };
    if found.next().is_some() {
        return Err("发现多个 CHERRY PIXIU75 电量接口，停止查询".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    io.write(&request(), Duration::from_millis(500))?;
    let until = std::time::Instant::now() + Duration::from_millis(1000);
    while std::time::Instant::now() < until {
        let reply = io.read(REPORT_LEN, Duration::from_millis(180))?;
        if let Some((percent, charging)) = parse_reply(&reply) {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: interface.pid == 0x01e2,
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                identity: format!("CHERRY PIXIU75 · 046A:{:04X} · MI_01 Col04", interface.pid),
                settings_error: "仅适配已记录的 PIXIU75 块通道；其他键盘设置不在此插件内。".into(),
                ..State::default()
            });
        }
    }
    Err("CHERRY PIXIU75 未收到块号与长度均匹配的电量回复".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value =
            serde_json::from_str(input).map_err(|error| format!("请求 JSON 无效：{error}"))?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("该插件只读 PIXIU75 电量与充电状态，不写键位、宏或灯光".to_string());
        }
        read_state()
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("cherry-pixiu75", handler);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_matching_battery_block_and_known_charge_bit() {
        let mut f = [0; REPORT_LEN];
        f[0] = REPORT;
        f[3] = BLOCK;
        f[4] = 4;
        f[8] = 48;
        assert_eq!(parse_reply(&f), Some((48, false)));
        f[9] = 1;
        assert_eq!(parse_reply(&f), Some((48, true)));
        f[9] = 3;
        assert_eq!(parse_reply(&f), None);
        f[9] = 0;
        f[3] = 0x1b;
        assert_eq!(parse_reply(&f), None);
    }
}
