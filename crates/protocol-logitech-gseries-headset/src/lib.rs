//! HID++ 2.0 headset battery facts from HaloBattery/providers/logitech.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Spec {
    pub key: &'static str,
    pub model: &'static str,
    pub direct_pids: &'static [u16],
}

const RECEIVERS: &[u16] = &[
    0xc52b, 0xc532, 0xc52e, 0xc531, 0xc534, 0xc535, 0xc537, 0xc539, 0xc53a, 0xc53d, 0xc53f, 0xc541,
    0xc545, 0xc547, 0xc54d,
];
const LEN: usize = 20;

fn candidate(i: &hid::Interface, spec: &Spec) -> bool {
    if i.vid != 0x046d || i.input_len as usize != LEN || i.output_len as usize != LEN {
        return false;
    }
    let receiver = RECEIVERS.contains(&i.pid);
    (receiver && i.usage_page == 0xff00 && i.usage == 2)
        || (spec.direct_pids.contains(&i.pid)
            && ((i.usage_page == 0xff43 && i.usage == 0x0202)
                || (i.pid == 0x0ac4 && i.usage_page == 0x000c && i.usage == 1)
                || (i.usage_page == 0xff00 && i.usage == 2)))
}

fn exchange(
    io: &mut WinTransport,
    slot: u8,
    feature: u8,
    function: u8,
    params: &[u8],
) -> Result<Vec<u8>, String> {
    let mut req = [0u8; LEN];
    req[..4].copy_from_slice(&[0x11, slot, feature, function | 0x0b]);
    req[4..4 + params.len()].copy_from_slice(params);
    io.flush()?;
    io.write(&req, Duration::from_millis(500))?;
    let until = Instant::now() + Duration::from_millis(600);
    while Instant::now() < until {
        let reply = io.read(LEN, Duration::from_millis(100))?;
        if reply.len() < 6 || reply[1] != slot {
            continue;
        }
        if matches!(reply[2], 0x8f | 0xff) && reply[3..5] == req[2..4] {
            return Err(format!("HID++ 返回错误 0x{:02X}", reply[5]));
        }
        if reply[0] == 0x11 && reply.len() >= LEN && reply[2..4] == req[2..4] {
            return Ok(reply[4..].to_vec());
        }
    }
    Err("HID++ 查询超时".into())
}

fn feature(io: &mut WinTransport, slot: u8, id: u16) -> Result<u8, String> {
    let r = exchange(io, slot, 0, 0, &id.to_be_bytes())?;
    r.first()
        .copied()
        .filter(|x| *x != 0)
        .ok_or_else(|| format!("缺少 HID++ 特性 {id:04X}"))
}

fn name(io: &mut WinTransport, slot: u8) -> Result<String, String> {
    let idx = feature(io, slot, 0x0005)?;
    let len = exchange(io, slot, idx, 0, &[])?[0] as usize;
    if !(1..=64).contains(&len) {
        return Err("HID++ 设备名长度无效".into());
    }
    let mut bytes = Vec::with_capacity(len);
    while bytes.len() < len {
        let part = exchange(io, slot, idx, 0x10, &[bytes.len() as u8])?;
        bytes.extend_from_slice(&part[..(len - bytes.len()).min(part.len())]);
    }
    String::from_utf8(bytes).map_err(|_| "HID++ 设备名无效".into())
}

fn exact_model(name: &str, model: &str) -> bool {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token.eq_ignore_ascii_case(model))
}

const CURVE: &[(u16, u8)] = &[
    (4186, 100),
    (4067, 90),
    (3989, 80),
    (3922, 70),
    (3859, 60),
    (3811, 50),
    (3778, 40),
    (3751, 30),
    (3717, 20),
    (3671, 10),
    (3646, 5),
    (3579, 2),
    (3500, 0),
];

pub fn parse_adc(params: &[u8]) -> Option<(u8, bool)> {
    if params.len() < 3 || params[2] & 1 == 0 {
        return None;
    }
    let mv = u16::from_be_bytes([params[0], params[1]]);
    if mv < 2500 {
        return None;
    }
    let percent = if mv >= CURVE[0].0 {
        100
    } else {
        CURVE
            .windows(2)
            .find_map(|pair| {
                let (hi_mv, hi_percent) = pair[0];
                let (lo_mv, lo_percent) = pair[1];
                (mv >= lo_mv).then(|| {
                    let delta = (mv - lo_mv) as u32 * (hi_percent - lo_percent) as u32;
                    lo_percent
                        + ((delta + ((hi_mv - lo_mv) / 2) as u32) / (hi_mv - lo_mv) as u32) as u8
                })
            })
            .unwrap_or(0)
    };
    Some((percent, params[2] & 2 != 0))
}

pub fn read_state(spec: &Spec) -> Result<State, String> {
    for i in hid::enumerate()?.iter().filter(|i| candidate(i, spec)) {
        let receiver = RECEIVERS.contains(&i.pid);
        let mut io = WinTransport::open(&i.path)?;
        for slot in if receiver {
            (1..=6).collect::<Vec<_>>()
        } else {
            vec![0xff]
        } {
            let Ok(device_name) = name(&mut io, slot) else {
                continue;
            };
            if !exact_model(&device_name, spec.model) {
                continue;
            }
            let adc = feature(&mut io, slot, 0x1f20)?;
            let (percent, charging) = parse_adc(&exchange(&mut io, slot, adc, 0, &[])?)
                .ok_or("耳机已识别，但未取得有效电量")?;
            return Ok(State {
                key: spec.key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: false,
                identity: format!("{device_name} · 046D:{:04X} · slot {slot}", i.pid),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅支持只读电量；未经本项目实机验证。".into(),
                ..State::default()
            });
        }
    }
    Ok(State {
        key: spec.key.into(),
        error: format!("未连接 {}（设备名不匹配时不显示）", spec.model),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adc_and_identity() {
        assert_eq!(parse_adc(&[0x0f, 0x95, 3]), Some((80, true)));
        assert_eq!(parse_adc(&[0x0f, 0x95, 0]), None);
        assert!(exact_model("Logitech G733 LIGHTSPEED", "G733"));
        assert!(!exact_model("Logitech G733X", "G733"));
    }
}
