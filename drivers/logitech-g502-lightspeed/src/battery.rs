//! Read-only HID++ feature lookups. Solaar documents device-name and battery features.
use peripheral_desk::{hid, model::State};
use peripheral_desk_protocol_hidpp::{transport::WinTransport, Transport};
use std::time::{SystemTime, UNIX_EPOCH};

fn call(
    io: &mut impl Transport,
    feature: u8,
    function: u8,
    args: &[u8],
) -> Result<Vec<u8>, String> {
    let mut req = [0u8; 20];
    req[..4].copy_from_slice(&[0x11, 1, feature, function | 0x0c]);
    req[4..4 + args.len()].copy_from_slice(args);
    io.exchange(&req)
}
fn feature(io: &mut impl Transport, code: u16) -> Result<u8, String> {
    let reply = call(io, 0, 0, &code.to_be_bytes())?;
    match reply.first() {
        Some(0) | None => Err(format!("HID++ 未提供特性 {code:04X}")),
        Some(v) => Ok(*v),
    }
}
fn name(io: &mut impl Transport) -> Result<String, String> {
    let index = feature(io, 5)?;
    let count = *call(io, index, 0, &[])?.first().ok_or("设备名长度缺失")? as usize;
    if count == 0 || count > 64 {
        return Err("设备名长度无效".into());
    }
    let mut data = Vec::with_capacity(count);
    while data.len() < count {
        let chunk = call(io, index, 0x10, &[data.len() as u8])?;
        if chunk.is_empty() {
            return Err("设备名片段为空".into());
        }
        data.extend_from_slice(&chunk[..chunk.len().min(count - data.len())]);
    }
    String::from_utf8(data).map_err(|e| e.to_string())
}
fn battery(io: &mut impl Transport) -> Result<(u8, Option<bool>), String> {
    if let Ok(index) = feature(io, 0x1004) {
        let reply = call(io, index, 0x10, &[])?;
        if let Some(level @ 0..=100) = reply.first().copied() {
            return Ok((level, reply.get(1).map(|v| v & 2 != 0)));
        }
    }
    if let Ok(index) = feature(io, 0x1000) {
        let reply = call(io, index, 0, &[])?;
        if let (Some(level @ 0..=100), Some(status)) = (reply.first().copied(), reply.get(2)) {
            return Ok((level, Some((1..=4).contains(status))));
        }
    }
    let index = feature(io, 0x1001)?;
    let reply = call(io, index, 0, &[])?;
    let mv = u16::from_be_bytes([
        *reply.first().ok_or("电压回复缺失")?,
        *reply.get(1).ok_or("电压回复不完整")?,
    ]);
    // Coarse Li-ion voltage curve used for devices exposing voltage only.
    let points: [(u16, u8); 9] = [
        (3300, 0),
        (3500, 5),
        (3600, 10),
        (3700, 25),
        (3800, 45),
        (3900, 65),
        (4000, 82),
        (4100, 95),
        (4200, 100),
    ];
    let mut level = 100u8;
    for pair in points.windows(2) {
        if mv <= pair[1].0 {
            level = pair[0].1
                + (mv.saturating_sub(pair[0].0) as u32 * (pair[1].1 - pair[0].1) as u32
                    / (pair[1].0 - pair[0].0) as u32) as u8;
            break;
        }
    }
    Ok((level, None))
}
pub fn read_state(key: &str, expected: &str) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let mut last = format!("未找到 {expected} 对应的 HID++ 设备");
    for i in interfaces.iter().filter(|i| {
        i.vid == 0x046d
            && matches!(i.pid, 0xc539 | 0xc547)
            && i.usage_page == 0xff00
            && i.usage == 2
            && i.input_len == 20
            && i.output_len == 20
    }) {
        let result = (|| {
            let mut io = WinTransport::open(&i.path)?;
            let actual = name(&mut io)?;
            if !actual.to_ascii_uppercase().contains(expected) {
                return Err(format!("设备名不符：{actual}"));
            }
            let (percent, charging) = battery(&mut io)?;
            Ok(State {
                key: key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging,
                identity: format!("Logitech {actual} · 046D:{:04X}", i.pid),
                settings_error: "仅支持只读电量。".into(),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..State::default()
            })
        })();
        match result {
            Ok(state) => return Ok(state),
            Err(e) => last = e,
        }
    }
    Err(last)
}
