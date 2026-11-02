//! Scyrox V6/V8S battery frames from public reverse-engineering projects.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub struct Model {
    pub key: &'static str,
    pub name: &'static str,
    pub wired_pid: u16,
    pub receiver_pid: u16,
}

const VID: u16 = 0x3554;
const REPORT_ID: u8 = 0x08;
const REPORT_LEN: usize = 17;

pub fn checksum(frame: &[u8]) -> u8 {
    0x55u8.wrapping_sub(
        frame[..REPORT_LEN - 1]
            .iter()
            .fold(0, |sum, byte| sum.wrapping_add(*byte)),
    )
}

pub fn request(command: u8) -> Option<[u8; REPORT_LEN]> {
    if !matches!(command, 0x03 | 0x04) {
        return None;
    }
    let mut frame = [0; REPORT_LEN];
    frame[0] = REPORT_ID;
    frame[1] = command;
    frame[16] = checksum(&frame);
    Some(frame)
}

pub fn parse_reply(frame: &[u8], command: u8) -> Option<&[u8]> {
    (frame.len() == REPORT_LEN
        && frame[0] == REPORT_ID
        && frame[1] == command
        && frame[16] == checksum(frame))
    .then_some(frame)
}

fn exchange(transport: &mut WinTransport, command: u8) -> Result<Vec<u8>, String> {
    let request = request(command).ok_or("未知 Scyrox 查询命令")?;
    transport.write(&request, Duration::from_millis(500))?;
    let until = Instant::now() + Duration::from_millis(900);
    while Instant::now() < until {
        let response = transport.read(REPORT_LEN, Duration::from_millis(120))?;
        if parse_reply(&response, command).is_some() {
            return Ok(response);
        }
    }
    Err("Scyrox 未收到匹配的 HID 回复；鼠标可能休眠".into())
}

fn supported(interface: &hid::Interface, model: Model) -> bool {
    interface.vid == VID
        && matches!(interface.pid, pid if pid == model.wired_pid || pid == model.receiver_pid)
        && interface.input_len as usize == REPORT_LEN
        && interface.output_len as usize == REPORT_LEN
}

pub fn read_state(model: Model) -> Result<State, String> {
    let interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|interface| supported(interface, model))
        .collect();
    if interfaces.is_empty() {
        return Ok(State {
            key: model.key.into(),
            error: "未找到 Scyrox 指定型号的 17 字节 HID 接口".into(),
            ..State::default()
        });
    }

    let mut first_error = None;
    for interface in interfaces {
        let result = (|| {
            let mut transport = WinTransport::open(&interface.path)?;
            transport.flush()?;
            if interface.pid == model.receiver_pid {
                let online = exchange(&mut transport, 0x03)?;
                if online[6] > 1 {
                    return Err("Scyrox 在线状态字段无效".into());
                }
                if online[6] == 0 {
                    return Ok(State {
                        key: model.key.into(),
                        present: true,
                        fresh: true,
                        identity: format!(
                            "{} · 3554:{:04X} · 鼠标休眠或离线",
                            model.name, interface.pid
                        ),
                        settings_error: "接收器在线；鼠标未响应，不提供过期电量。".into(),
                        updated: now(),
                        ..State::default()
                    });
                }
            }
            let battery = exchange(&mut transport, 0x04)?;
            let percent = battery[6];
            if percent > 100 || battery[7] > 1 {
                return Err("Scyrox 电量回复包含无效字段".into());
            }
            let voltage = u16::from_be_bytes([battery[8], battery[9]]);
            Ok(State {
                key: model.key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(battery[7] == 1),
                wired: interface.pid == model.wired_pid,
                identity: format!("{} · 3554:{:04X} · {voltage} mV", model.name, interface.pid),
                settings_error: "仅提供只读电量、充电状态和上游回复的电压。".into(),
                updated: now(),
                ..State::default()
            })
        })();
        match result {
            Ok(state) => return Ok(state),
            Err(error) if first_error.is_none() => first_error = Some(error),
            Err(_) => {}
        }
    }
    Err(first_error.unwrap_or_else(|| "Scyrox 无有效电量回复".into()))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_exact_frame_shape_command_checksum_and_values() {
        let mut frame = [
            0x08, 0x04, 0, 0, 0, 0x02, 0x14, 0, 0x0e, 0x18, 0, 0, 0, 0, 0, 0, 0x0d,
        ];
        assert_eq!(request(0x04).unwrap()[16], 0x49);
        assert!(parse_reply(&frame, 0x04).is_some());
        assert!(parse_reply(&frame[..16], 0x04).is_none());
        frame[16] ^= 1;
        assert!(parse_reply(&frame, 0x04).is_none());
        assert!(request(0x05).is_none());
    }
}
