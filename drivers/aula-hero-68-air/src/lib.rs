//! AULA HERO 68 Air receiver protocol adapter from Vigil Watt v1.3.0 (a78e973).
//! The Rust implementation is an adaptation of its documented MIT protocol.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const KEY: &str = "aula-hero-68-air";
const VID: u16 = 0x372e;
const PID: u16 = 0x106c;
const REPORT: u8 = 0x09;
const REPORT_LEN: usize = 20;
const PACKET_LEN: usize = 63;
const PART: usize = 14;

fn matches(i: &hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == 0xff60
        && i.usage == 0x0061
        && i.input_len as usize == REPORT_LEN
        && i.output_len as usize == REPORT_LEN
}
fn sum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |s, b| s.wrapping_add(*b))
}

fn query(command: u8, parameter: u8, length: u8) -> [u8; 64] {
    let mut p = [0; 64];
    p[0] = REPORT;
    p[1] = command;
    p[2] = parameter;
    p[4] = 1;
    p[6] = length;
    p[63] = 255u8.wrapping_sub(sum(&p[..63]));
    p
}

fn fragment(payload: &[u8], total: u8, current: u8, sync: u8) -> Result<[u8; 20], String> {
    if payload.is_empty()
        || payload.len() > PART
        || !(1..=5).contains(&total)
        || current == 0
        || current > total
        || sync > 7
    {
        return Err("HERO 68 Air 分片参数越界".into());
    }
    let mut p = [0; 20];
    p[0] = REPORT;
    p[1] = 0x66;
    p[2] = total | ((sync & 4) << 5);
    p[3] = current | ((sync & 2) << 6);
    p[4] = payload.len() as u8 | ((sync & 1) << 7);
    p[5..5 + payload.len()].copy_from_slice(payload);
    p[5 + payload.len()] = sum(&p[1..5 + payload.len()]);
    Ok(p)
}

fn fragment_info(p: &[u8]) -> Option<(u8, u8, u8, usize)> {
    if p.len() != REPORT_LEN || p[0] != REPORT || p[1] != 0x66 {
        return None;
    }
    let total = p[2] & 0x7f;
    let current = p[3] & 0x7f;
    let len = (p[4] & 0x7f) as usize;
    let sync = ((p[2] & 0x80) >> 5) | ((p[3] & 0x80) >> 6) | ((p[4] & 0x80) >> 7);
    if !(1..=5).contains(&total)
        || current == 0
        || current > total
        || len > PART
        || p[5 + len] != sum(&p[1..5 + len])
    {
        return None;
    }
    Some((total, current, sync, len))
}

fn ack(p: &[u8]) -> Result<[u8; 20], String> {
    let (_, _, _, len) = fragment_info(p).ok_or("HERO 68 Air 分片校验失败")?;
    if len == 0 {
        return Err("HERO 68 Air 空分片不能应答".into());
    }
    let mut a = [0; 20];
    a[..5].copy_from_slice(&p[..5]);
    a[4] &= 0x80;
    a[5] = sum(&a[1..5]);
    Ok(a)
}

fn is_ack(p: &[u8], sent: &[u8; 20]) -> bool {
    matches!(fragment_info(p), Some((_, _, _, 0)))
        && p[2] == sent[2]
        && p[3] == sent[3]
        && p[4] == (sent[4] & 0x80)
}

struct Assembler {
    packet: [u8; PACKET_LEN],
    total: u8,
    current: u8,
    sync: u8,
    len: usize,
}
impl Default for Assembler {
    fn default() -> Self {
        Self {
            packet: [0; PACKET_LEN],
            total: 0,
            current: 0,
            sync: 0,
            len: 0,
        }
    }
}
impl Assembler {
    fn reset(&mut self) {
        *self = Self::default();
    }
    fn accept(&mut self, p: &[u8]) -> Option<[u8; PACKET_LEN]> {
        let (total, current, sync, len) = fragment_info(p)?;
        if len == 0 {
            return None;
        }
        if current == self.current && sync == self.sync && total == self.total {
            return None;
        }
        if current == 1 {
            self.reset();
            self.total = total;
        }
        if total != self.total
            || current != self.current + 1
            || self.len + len > PACKET_LEN
            || (current > 1 && sync != ((self.sync + 1) & 7))
        {
            self.reset();
            return None;
        }
        self.packet[self.len..self.len + len].copy_from_slice(&p[5..5 + len]);
        self.len += len;
        self.current = current;
        self.sync = sync;
        (current == total && self.len == PACKET_LEN).then_some(self.packet)
    }
}

fn matching_response(p: &[u8; PACKET_LEN], command: u8, parameter: u8, length: u8) -> bool {
    p[0] == command
        && p[1] == parameter
        && p[2] == 0
        && p[3] == 1
        && p[4] == 0
        && p[5] == length
        && sum(p).wrapping_add(REPORT) == 0xff
}

fn transact(
    io: &mut WinTransport,
    bytes: &[u8; 64],
    command: u8,
    parameter: u8,
    length: u8,
    sync: &mut u8,
) -> Result<[u8; PACKET_LEN], String> {
    let mut assembler = Assembler::default();
    let started = Instant::now();
    let mut assembled = None;
    for index in 0..5 {
        if started.elapsed() > Duration::from_secs(3) {
            return Err("HERO 68 Air 查询超时".into());
        }
        let offset = index * PART;
        let count = PART.min(bytes.len() - offset);
        *sync = (*sync + 1) & 7;
        let sent = fragment(&bytes[offset..offset + count], 5, (index + 1) as u8, *sync)?;
        io.write(&sent, Duration::from_millis(500))?;
        let until = Instant::now() + Duration::from_millis(500);
        let mut acknowledged = false;
        while Instant::now() < until {
            let input = io.read(REPORT_LEN, Duration::from_millis(100))?;
            if is_ack(&input, &sent) {
                acknowledged = true;
                break;
            }
            if let Some((_, _, _, len)) = fragment_info(&input) {
                if len > 0 {
                    io.write(&ack(&input)?, Duration::from_millis(500))?;
                    if let Some(packet) = assembler.accept(&input) {
                        if matching_response(&packet, command, parameter, length) {
                            assembled = Some(packet);
                        }
                    }
                }
            }
        }
        if !acknowledged {
            return Err("HERO 68 Air 分片未收到匹配 ACK".into());
        }
    }
    let until = Instant::now() + Duration::from_secs(3).saturating_sub(started.elapsed());
    while assembled.is_none() && Instant::now() < until {
        let input = io.read(REPORT_LEN, Duration::from_millis(100))?;
        if let Some((_, _, _, len)) = fragment_info(&input) {
            if len > 0 {
                io.write(&ack(&input)?, Duration::from_millis(500))?;
                if let Some(packet) = assembler.accept(&input) {
                    if matching_response(&packet, command, parameter, length) {
                        assembled = Some(packet);
                    }
                }
            }
        }
    }
    assembled.ok_or_else(|| "HERO 68 Air 未收到完整且匹配的查询回复".into())
}

fn read_state() -> Result<State, String> {
    let ports = hid::enumerate()?;
    let found: Vec<_> = ports.iter().filter(|i| matches(i)).collect();
    if found.len() != 1 {
        return Err("未找到唯一的 AULA HERO 68 Air 接收器通道".into());
    }
    let port = found[0];
    let mut io = WinTransport::open(&port.path)?;
    io.flush()?;
    let mut sync = 0;
    let uuid = transact(&mut io, &query(0x82, 1, 6), 0x82, 1, 6, &mut sync)?;
    if uuid[6..12] != [0x12, 0, 0, 0, 0, 0x0a] {
        return Err("接收器 UUID 不匹配 AULA HERO 68 Air，已停止读取".into());
    }
    let reply = transact(&mut io, &query(0x87, 0, 2), 0x87, 0, 2, &mut sync)?;
    if reply[6] > 100 {
        return Err("AULA HERO 68 Air 电量字段超出 0–100".into());
    }
    let charge = match reply[7] & 0xf0 != 0 {
        true => Some(true),
        false if reply[7] & 0x0f != 0 => Some(true),
        _ => Some(false),
    };
    let mut state = State::default();
    state.key = KEY.into();
    state.present = true;
    state.fresh = true;
    state.percent = Some(reply[6]);
    state.charging = charge;
    state.wired = false;
    state.identity = "AULA HERO 68 Air · 接收器 UUID 已确认".into();
    state.updated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(state)
}

fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        let request: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        match request.get("op").and_then(serde_json::Value::as_str) {
            Some("read-state") => serde_json::to_value(read_state()?).map_err(|e| e.to_string()),
            _ => Err("AULA HERO 68 Air 插件提供接收器电量与充电状态；灯光、改键和宏未接入".into()),
        }
    })();
    match result {
        Ok(value) => serde_json::json!({"ok":true,"value":value,"error":""}),
        Err(error) => serde_json::json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

peripheral_desk_driver_ffi::export_driver!("aula-hero-68-air", handler);
