use peripheral_desk::{hid, model::State};
use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const KEY: &str = "royal-kludge-m3";
static SEQUENCE: AtomicU8 = AtomicU8::new(1);

fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let Some(i) = interfaces
        .iter()
        .filter(|i| {
            i.vid == 0x372e
                && matches!(i.pid, 0x1019 | 0x102a)
                && i.usage_page == 0xff00
                && i.feature_len >= 16
        })
        .max_by_key(|i| i.pid == 0x102a)
    else {
        return Ok(State {
            key: KEY.into(),
            error: "未找到 RK M3 接收器或线缆接口".into(),
            ..State::default()
        });
    };
    let device = hid::Device::open(&i.path)?;
    let sn = SEQUENCE.fetch_add(1, Ordering::Relaxed).wrapping_rem(200) + 1;
    let mut frame = vec![0; i.feature_len as usize];
    frame[0] = 3;
    frame[2..7].copy_from_slice(&[0x50, 0, sn, 0x4f, 0x81]);
    frame[1] = frame[2..frame.len().min(63)]
        .iter()
        .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
    device.send(&frame)?;
    std::thread::sleep(Duration::from_millis(25));
    for _ in 0..12 {
        let reply = device.receive(3, frame.len())?;
        if reply.len() >= 7
            && reply[0] == 3
            && reply[1] == 0x50
            && reply[2] == 0
            && reply[4] == sn
            && reply[5] == 1
            && (reply[6] & 0x7f) <= 100
        {
            return Ok(State {
                key: KEY.into(),
                present: true,
                fresh: true,
                percent: Some(reply[6] & 0x7f),
                charging: Some(reply[6] & 0x80 != 0),
                identity: format!("Royal Kludge M3 · 372E:{:04X}", i.pid),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅提供只读电量与充电状态。".into(),
                ..State::default()
            });
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    Err("RK M3 未返回电量回复（可能休眠）".into())
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if request.get("op").and_then(serde_json::Value::as_str) != Some("read-state") {
            return Err("仅支持只读电量".into());
        }
        serde_json::to_value(read_state()?).map_err(|e| e.to_string())
    })();
    match result {
        Ok(v) => serde_json::json!({"ok":true,"value":v,"error":""}),
        Err(e) => serde_json::json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("royal-kludge-m3", handler);
