//! HECATE 3-slot wireless receiver battery reader, adapted from Vigil Watt v1.3.0.
//! Receiver protocol does not identify the paired mouse model; see license notice.
use peripheral_desk::{
    hid::{self, Interface},
    model::State,
};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_KEY: &str = "hecate-g3m-pro-upgrade";
const VID: u16 = 0x35BB;
const PID: u16 = 0xD3FB;
const PAGE: u16 = 0xFF02;
const USAGE: u16 = 0x0002;
const LEN: usize = 17;
const RID: u8 = 0x08;
const STATUS: u8 = 0x03;
const POWER: u8 = 0x04;

pub fn query(command: u8) -> Option<[u8; LEN]> {
    if command != STATUS && command != POWER {
        return None;
    }
    let mut f = [0; LEN];
    f[0] = RID;
    f[1] = command;
    let sum: u16 = f[..LEN - 1].iter().map(|b| *b as u16).sum();
    f[LEN - 1] = (0x55u16.wrapping_sub(sum) & 0xff) as u8;
    Some(f)
}
pub fn parse_reply(frame: &[u8], command: u8) -> Option<&[u8]> {
    if frame.len() != LEN || frame[0] != RID || frame[1] != command {
        return None;
    }
    let sum: u16 = frame[..LEN - 1].iter().map(|b| *b as u16).sum();
    (frame[LEN - 1] == (0x55u16.wrapping_sub(sum) & 0xff) as u8).then_some(frame)
}
fn supported(i: &Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == PAGE
        && i.usage == USAGE
        && i.input_len as usize == LEN
        && i.output_len as usize == LEN
}
fn query_one(io: &mut WinTransport, command: u8) -> Result<[u8; LEN], String> {
    io.flush()?;
    let request = query(command).ok_or("未知 HECATE 查询命令")?;
    io.write(&request, Duration::from_millis(500))?;
    let deadline = Instant::now() + Duration::from_millis(1200);
    while Instant::now() < deadline {
        let reply = io.read(LEN, Duration::from_millis(200))?;
        if let Some(bytes) = parse_reply(&reply, command) {
            let mut frame = [0; LEN];
            frame.copy_from_slice(bytes);
            return Ok(frame);
        }
    }
    Err("HECATE 接收器未返回匹配查询帧".into())
}
pub fn read_state() -> Result<State, String> {
    let devices = hid::enumerate()?;
    let mut matches = devices.iter().filter(|i| supported(i));
    let Some(i) = matches.next() else {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "未找到 HECATE 35BB:D3FB 17-byte 接收器接口".into(),
            settings_error: "该接收器协议不含鼠标型号；充电字段仍未验证。".into(),
            ..State::default()
        });
    };
    if matches.next().is_some() {
        return Err("发现多个 HECATE D3FB 电量接口，已停止查询".into());
    }
    let mut io = WinTransport::open(&i.path)?;
    let status = query_one(&mut io, STATUS)?;
    if status[6] != 1 {
        return Ok(State {
            key: DEVICE_KEY.into(),
            error: "HECATE D3FB 接收器未报告已连接的鼠标".into(),
            settings_error: "接收器协议不返回型号；充电状态未验证。".into(),
            ..State::default()
        });
    }
    let power = query_one(&mut io, POWER)?;
    let percent = power[6];
    if percent > 100 {
        return Err("HECATE D3FB 返回超范围电量字段".into());
    }
    if percent == 0 {
        return Err("HECATE D3FB 返回零值占位，电量按未知处理".into());
    }
    let mv = u16::from_be_bytes([power[8], power[9]]);
    Ok(State { key: DEVICE_KEY.into(), present: true, fresh: true, percent: Some(percent), charging: None, updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(), identity: format!("HECATE D3FB 3-slot 接收器 · 35BB:D3FB · FF02 · {mv} mV"), settings_error: "只读电量。接收器帧未标出型号；G3M Pro 升级版适配依据为 Vigil Watt 用户设备记录，充电状态/其他控制未验证。".into(), ..State::default() })
}
