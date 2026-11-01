//! Read-only G522 LIGHTSPEED Centurion bridge exchange.
//! Protocol facts: HeadsetControl 7b760ddb29e4618f4229ab2936a4b009ab2e1fa0,
//! `logitech_g522_lightspeed.hpp` and `logitech_centurion_protocol.hpp` (GPL-3.0).

use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LEN: usize = 64;
const PREFIX: [u8; 2] = [0x50, 0x23];
const IO_TIMEOUT: Duration = Duration::from_millis(500);

fn frame(payload: &[u8]) -> [u8; LEN] {
    let mut wire = [0; LEN];
    wire[..2].copy_from_slice(&PREFIX);
    wire[2] = (payload.len() + 1) as u8;
    wire[4..4 + payload.len()].copy_from_slice(payload);
    wire
}

fn payload(wire: &[u8]) -> Option<&[u8]> {
    if wire.len() < 6 || wire[..2] != PREFIX || wire[2] < 2 {
        return None;
    }
    let end = 3 + wire[2] as usize;
    wire.get(4..end)
}

fn direct(
    io: &mut WinTransport,
    feature: u8,
    function: u8,
    params: &[u8],
) -> Result<Vec<u8>, String> {
    let mut request = vec![feature, function | 1];
    request.extend_from_slice(params);
    io.write(&frame(&request), IO_TIMEOUT)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        let reply = io.read(LEN, Duration::from_millis(250))?;
        if let Some(body) = payload(&reply) {
            if body.len() >= 2 && body[0] == feature && body[1] == (function | 1) {
                return Ok(body[2..].to_vec());
            }
        }
    }
    Err("G522 接收器特性查询超时".into())
}

fn bridged(
    io: &mut WinTransport,
    bridge: u8,
    feature: u8,
    params: &[u8],
) -> Result<Vec<u8>, String> {
    let mut request = vec![bridge, 0x11, 0, (params.len() + 3) as u8, 0, feature, 1];
    request.extend_from_slice(params);
    io.write(&frame(&request), IO_TIMEOUT)?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut acknowledged = false;
    while Instant::now() < deadline {
        let reply = io.read(LEN, Duration::from_millis(250))?;
        let Some(body) = payload(&reply) else {
            continue;
        };
        if body.first().copied() != Some(bridge) {
            continue;
        }
        if body.len() >= 4 && body[1] == 0 && body[3] == 0 {
            return Err("G522 耳机未连接".into());
        }
        if body.get(1) == Some(&0x11) {
            acknowledged = true;
            continue;
        }
        if body.len() < 7 || body[1] != 0x10 || body[4] != 0 {
            continue;
        }
        if body[5] == 0xff && body[6] == feature {
            return Err("G522 耳机拒绝特性查询".into());
        }
        if body[5] == feature && body[6] == 1 {
            return Ok(body[7..].to_vec());
        }
    }
    Err(if acknowledged {
        "G522 耳机响应超时"
    } else {
        "G522 接收器未确认请求"
    }
    .into())
}

pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let matches: Vec<_> = interfaces
        .iter()
        .filter(|i| {
            i.vid == 0x046d
                && i.pid == 0x0b18
                && i.usage_page == 0xffa0
                && i.usage == 1
                && i.input_len as usize >= LEN
                && i.output_len as usize >= LEN
        })
        .collect();
    let interface = match matches.as_slice() {
        [] => {
            return Ok(State {
                key: "logitech-g522-lightspeed".into(),
                error: "未找到 G522 LIGHTSPEED 接收器通道".into(),
                ..State::default()
            })
        }
        [interface] => interface,
        _ => return Err("发现多个 G522 Centurion 通道".into()),
    };
    let mut io = WinTransport::open(&interface.path)?;
    io.flush()?;
    let bridge = direct(&mut io, 0, 0, &[0, 3])?
        .first()
        .copied()
        .filter(|x| *x != 0)
        .ok_or("G522 接收器缺少桥接特性")?;
    let battery_feature = bridged(&mut io, bridge, 0, &[1, 4])?
        .first()
        .copied()
        .filter(|x| *x != 0)
        .ok_or("G522 耳机缺少电量特性")?;
    let battery = bridged(&mut io, bridge, battery_feature, &[])?;
    let percent = *battery
        .first()
        .filter(|x| **x <= 100)
        .ok_or("G522 电量回复无效")?;
    let charging = match battery.get(2) {
        Some(0) => Some(false),
        Some(1 | 2) => Some(true),
        _ => None,
    };
    Ok(State {
        key: "logitech-g522-lightspeed".into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging,
        wired: false,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: "Logitech G522 LIGHTSPEED · 046D:0B18 · FFA0:0001".into(),
        settings_error: "仅提供只读电量和充电状态。".into(),
        ..State::default()
    })
}
