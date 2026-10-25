//! Logitech Bolt receiver + exact paired device-name MX Master 3S battery reader.
//! Independent HID++ implementation; protocol facts and validation range are in UPSTREAM-SOURCES.md.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "logitech-bolt-mx-master-3s";
const VID: u16 = 0x046D;
const RECEIVER_PID: u16 = 0xC548;
const PAGE: u16 = 0xFF00;
const USAGE: u16 = 0x0002;
const REPORT: usize = 20;
const REPORT_ID: u8 = 0x11;
const SOFTWARE_ID: u8 = 0x0B;
const ROOT: u8 = 0;
const NAME_FEATURE: u16 = 0x0005;
const UNIFIED_BATTERY_FEATURE: u16 = 0x1004;
const MAX_BOLT_SLOTS: u8 = 6;
const TARGET_NAME: &str = "MX Master 3S";

fn feature_func(feature: u8, func: u8) -> [u8; 2] {
    [feature, (func << 4) | SOFTWARE_ID]
}
fn make_request(index: u8, feature: u8, func: u8, args: &[u8]) -> [u8; REPORT] {
    let mut frame = [0; REPORT];
    frame[0] = REPORT_ID;
    frame[1] = index;
    let [f, sw] = feature_func(feature, func);
    frame[2] = f;
    frame[3] = sw;
    let count = args.len().min(REPORT - 4);
    frame[4..4 + count].copy_from_slice(&args[..count]);
    frame
}
fn exchange(
    io: &mut WinTransport,
    request: &[u8; REPORT],
    timeout: Duration,
) -> Result<[u8; REPORT], String> {
    io.flush()?;
    io.write(request, Duration::from_millis(500))?;
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let wire = io.read(REPORT, Duration::from_millis(200))?;
        if wire.len() < REPORT || wire[0] != REPORT_ID {
            continue;
        }
        let dev_ok = wire[1] == request[1] || wire[1] == !request[1];
        if !dev_ok {
            continue;
        }
        if matches!(wire[2], 0x8F | 0xFF) && wire[3..5] == request[2..4] {
            return Err(format!("HID++ 设备拒绝查询（错误码 0x{:02X}）", wire[5]));
        }
        if wire[2..4] == request[2..4] {
            let mut exact = [0; REPORT];
            exact.copy_from_slice(&wire[..REPORT]);
            return Ok(exact);
        }
    }
    Err("HID++ Bolt 查询超时".into())
}
fn call(
    io: &mut WinTransport,
    index: u8,
    feature: u8,
    func: u8,
    args: &[u8],
    timeout: Duration,
) -> Result<[u8; REPORT], String> {
    exchange(io, &make_request(index, feature, func, args), timeout)
}
fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == RECEIVER_PID
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.input_len as usize == REPORT
        && i.output_len as usize == REPORT
        && i.feature_len == 0
}
fn discover_slot(io: &mut WinTransport, slot: u8) -> Result<Option<(String, u8)>, String> {
    // Root.Ping with this invocation's marker; 1.5 s matches Bolt's observed wake response window.
    let mark = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u8)
        .max(1);
    let ping = call(
        io,
        slot,
        ROOT,
        1,
        &[0, 0, mark],
        Duration::from_millis(1500),
    );
    let Ok(ping) = ping else { return Ok(None) };
    if ping[6] != mark {
        return Ok(None);
    }
    let Ok(feature) = call(
        io,
        slot,
        ROOT,
        0,
        &NAME_FEATURE.to_be_bytes(),
        Duration::from_millis(900),
    ) else {
        return Ok(None);
    };
    let name_index = feature[4];
    if name_index == 0 {
        return Ok(None);
    }
    let Ok(count) = call(io, slot, name_index, 0, &[], Duration::from_millis(900)) else {
        return Ok(None);
    };
    let length = count[4] as usize;
    if length == 0 || length > 32 {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(length);
    for offset in 0..length {
        let Ok(character) = call(
            io,
            slot,
            name_index,
            1,
            &[offset as u8],
            Duration::from_millis(900),
        ) else {
            return Ok(None);
        };
        bytes.push(character[4]);
    }
    let name = String::from_utf8_lossy(&bytes).trim().to_string();
    if name == TARGET_NAME {
        Ok(Some((name, ping[4])))
    } else {
        Ok(None)
    }
}
fn battery(io: &mut WinTransport, slot: u8) -> Result<(u8, Option<bool>), String> {
    let index = call(
        io,
        slot,
        ROOT,
        0,
        &UNIFIED_BATTERY_FEATURE.to_be_bytes(),
        Duration::from_millis(900),
    )?[4];
    if index == 0 {
        return Err("MX Master 3S 未公开 HID++ Unified Battery 0x1004".into());
    }
    // func 1 is GetStatus. Func 0 is GetCapabilities and must never be interpreted as a level.
    let response = call(io, slot, index, 1, &[], Duration::from_millis(900))?;
    let raw = response[4];
    if !(1..=100).contains(&raw) {
        return Err("MX Master 3S 返回不可用或超范围电量".into());
    }
    let charging = match response[6] {
        0 => Some(false),
        1 | 2 | 4 => Some(true),
        3 => Some(true),
        5 => return Err("设备报告电池异常".into()),
        6 => return Err("设备报告电池温度异常".into()),
        _ => None,
    };
    Ok((raw, charging))
}
pub fn read_state() -> Result<State, String> {
    let items = hid::enumerate()?;
    let mut candidates = items.iter().filter(|i| supported(i));
    let Some(interface) = candidates.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "未找到 Logitech Bolt C548 的 20-byte HID++ long 接口".into(),
            settings_error:
                "只适配 Bolt 接收器中的 MX Master 3S；蓝牙与其他罗技型号不包含在此插件。".into(),
            ..State::default()
        });
    };
    if candidates.next().is_some() {
        return Err("检测到多个匹配 Bolt 长接口，为避免重复查询已停止".into());
    }
    let mut io = WinTransport::open(&interface.path)?;
    let mut target = None;
    for slot in 1..=MAX_BOLT_SLOTS {
        if let Some(found) = discover_slot(&mut io, slot)? {
            target = Some((slot, found.0, found.1));
            break;
        }
    }
    let Some((slot, name, major)) = target else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "Bolt 配对槽中未找到设备名精确匹配 MX Master 3S".into(),
            settings_error: "未推断其他型号或连接方式；请确认鼠标已连接到此 Bolt 接收器。".into(),
            ..State::default()
        });
    };
    let (percent, charging) = battery(&mut io, slot)?;
    Ok(State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging,
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: format!("{name} · Logitech Bolt C548 · slot {slot} · HID++ {major}.?"),
        settings_error: "只读电量与充电状态；DPI、回报率、改键、宏、灯光和音效不在此插件范围。"
            .into(),
        ..State::default()
    })
}
