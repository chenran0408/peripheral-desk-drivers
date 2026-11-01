//! Hyzen dongle transaction, independently based on HaloBattery providers/lofree.py.
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LEN: usize = 32;
fn frame(command: u8) -> [u8; LEN] {
    let mut out = [0; LEN];
    out[0] = 4;
    out[3] = command;
    out
}
fn wait(io: &mut WinTransport, command: u8, deadline: Instant) -> Result<Vec<u8>, String> {
    while Instant::now() < deadline {
        let answer = io.read(LEN, Duration::from_millis(100))?;
        if answer.len() > 8
            && answer[0] == 4
            && (answer[3] == command || (command != 1 && command != 2 && answer[3] == 0))
        {
            return Ok(answer);
        }
    }
    Err(format!("Hyzen 事务 {command:02X} 超时"))
}
fn query(io: &mut WinTransport, command: u8) -> Result<u8, String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    io.write(&frame(1), Duration::from_millis(500))?;
    wait(io, 1, deadline)?;
    io.write(&frame(command), Duration::from_millis(500))?;
    let response = wait(io, command, deadline);
    io.write(&frame(2), Duration::from_millis(500))?;
    let _ = wait(io, 2, Instant::now() + Duration::from_millis(500));
    let response = response?;
    if response[5..7] != [0, 0] {
        return Err("Hyzen 回复偏移不匹配".into());
    }
    Ok(response[8])
}
pub fn read_state() -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    if interfaces
        .iter()
        .any(|i| i.vid == 0x388d && i.pid == 0x0024)
    {
        return Err("Hyzen 已接线，暂不读取接收器".into());
    }
    let i = interfaces
        .iter()
        .find(|i| {
            i.vid == 0x388d
                && i.pid == 0x0025
                && i.usage_page == 0xff1c
                && i.usage == 0x0092
                && i.output_len >= LEN as u16
        })
        .ok_or("未找到 Hyzen 2.4 GHz 接收器的控制集合")?;
    let mut io = WinTransport::open(&i.path)?;
    if query(&mut io, 0xaa)? == 0 {
        return Err("Hyzen 键盘离线".into());
    }
    let percent = query(&mut io, 0x1a)?;
    if percent > 100 {
        return Err("Hyzen 电量超出有效范围".into());
    }
    Ok(State {
        key: "lofree-hyzen".into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        identity: "Lofree Hyzen · 388D:0025 · 2.4 GHz".into(),
        settings_error: "仅接收器电量读取，不提供充电状态。".into(),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        ..State::default()
    })
}
