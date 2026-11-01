//! Keyboard battery exchange using OpenRazer's transaction/interface assignments.
//! The 91-byte Razer frame follows protocol-razer's read-only battery commands.
use peripheral_desk::{
    hid::{self, Device, Interface},
    model::State,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn checksum(wire: &[u8; 91]) -> u8 {
    wire[3..89].iter().fold(0, |sum, byte| sum ^ byte)
}

fn query(device: &Device, tid: u8, command: u8) -> Result<u8, String> {
    let mut req = [0u8; 91];
    req[2] = tid;
    req[6..9].copy_from_slice(&[2, 7, command]);
    req[89] = checksum(&req);
    device.send(&req)?;
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(60));
        let reply = device.receive(0, 91)?;
        let reply: &[u8; 91] = reply
            .as_slice()
            .try_into()
            .map_err(|_| "Razer 回复长度错误")?;
        if reply[0] != 0
            || reply[2] != tid
            || reply[5] != 0
            || reply[6] < 2
            || reply[7] != 7
            || reply[8] != command
            || reply[89] != checksum(reply)
        {
            continue;
        }
        match reply[1] {
            2 => return Ok(reply[10]),
            1 => continue,
            status => return Err(format!("Razer 未返回电量（状态 {status:02X}）")),
        }
    }
    Err("Razer 电量查询超时".into())
}

fn candidate(i: &Interface, pid: u16, number: u8) -> bool {
    i.vid == 0x1532
        && i.pid == pid
        && i.feature_len == 91
        && i.path
            .to_ascii_lowercase()
            .contains(&format!("mi_{number:02x}"))
}

pub fn read_state(
    key: &str,
    name: &str,
    wireless: (u16, u8),
    wired: (u16, u8, u8),
) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let found = interfaces.iter().filter_map(|i| {
        if candidate(i, wireless.0, wireless.1) {
            Some((i, wireless.0, 0x9f, false))
        } else if candidate(i, wired.0, wired.1) {
            Some((i, wired.0, wired.2, true))
        } else {
            None
        }
    });
    let mut error = format!("未找到 Razer {name} 指定控制接口");
    for (i, pid, tid, cable) in found {
        let attempt = (|| {
            let device = Device::open(&i.path)?;
            let raw = query(&device, tid, 0x80)?;
            let charging = query(&device, tid, 0x84).ok().map(|v| v != 0);
            Ok(State {
                key: key.into(),
                present: true,
                fresh: true,
                percent: Some(((u16::from(raw) * 100 + 127) / 255) as u8),
                battery_raw: Some(raw),
                charging,
                wired: cable,
                identity: format!("Razer {name} · 1532:{pid:04X} · TID {tid:02X}"),
                settings_error: "仅提供电量和充电状态读取。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            })
        })();
        match attempt {
            Ok(state) => return Ok(state),
            Err(e) => error = e,
        }
    }
    Err(error)
}
