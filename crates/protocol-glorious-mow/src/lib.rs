//! Model O/O-/D/D- Wireless battery protocol observed by korkje/mow (MIT).
use peripheral_desk::{hid, model::State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Model {
    pub key: &'static str,
    pub name: &'static str,
    pub wired: u16,
    pub receiver: u16,
}

pub fn parse(reply: &[u8]) -> Option<u8> {
    if reply.len() < 9 || reply[6] != 0x83 || reply[1] != 0xa1 {
        return None;
    }
    (1..=100).contains(&reply[8]).then_some(reply[8])
}

pub fn read_state(model: &Model) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let candidates: Vec<_> = interfaces
        .iter()
        .filter(|i| {
            i.vid == 0x258a
                && (i.pid == model.wired || i.pid == model.receiver)
                && i.feature_len == 65
        })
        .collect();
    if candidates.is_empty() {
        return Ok(State {
            key: model.key.into(),
            error: "未找到 Glorious 鼠标电量接口".into(),
            ..State::default()
        });
    }
    for i in candidates {
        let device = hid::Device::open(&i.path)?;
        let mut request = [0; 65];
        request[3] = 2;
        request[4] = 2;
        request[6] = 0x83;
        device.send(&request)?;
        std::thread::sleep(Duration::from_millis(50));
        let reply = device.receive(0, 65)?;
        if let Some(percent) = parse(&reply) {
            return Ok(State {
                key: model.key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                identity: format!("{} · 258A:{:04X}", model.name, i.pid),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅提供只读电量；HID 电量报告不包含可靠充电状态。".into(),
                ..State::default()
            });
        }
    }
    Err("Glorious 鼠标没有返回当前电量（可能休眠）".into())
}
