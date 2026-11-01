//! Read-only GNP battery query from HeadsetControl's Jabra protocol facts (GPL-3.0).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub enum Model {
    Evolve2Flex,
    Link380,
    Link390,
}

impl Model {
    fn spec(self) -> (&'static str, &'static str, u16, u16, u8) {
        match self {
            Self::Evolve2Flex => (
                "jabra-evolve2-65-flex",
                "Jabra Evolve2 65 Flex",
                0x2519,
                1,
                8,
            ),
            Self::Link380 => ("jabra-link-380", "Jabra Link 380", 0x24c7, 1, 4),
            Self::Link390 => ("jabra-link-390", "Jabra Link 390", 0x2e57, 5, 4),
        }
    }
}

pub fn parse_battery(reply: &[u8], address: u8, sequence: u8) -> Option<(u8, bool)> {
    if reply.len() < 9
        || reply[0] != 5
        || reply[1] != 0
        || reply[2] != address
        || reply[3] != sequence
        || reply[4] >> 6 != 3
        || reply[4] & 0x3f < 8
        || reply[5] != 0xff
        || reply[8] > 100
    {
        return None;
    }
    Some((reply[8], reply[7] & 1 != 0))
}

pub fn read_state(model: Model) -> Result<State, String> {
    let (key, name, pid, usage, addr) = model.spec();
    let Some(i) = hid::enumerate()?.into_iter().find(|i| {
        i.vid == 0x0b0e
            && i.pid == pid
            && i.usage_page == 0xff00
            && i.usage == usage
            && i.input_len >= 64
            && i.output_len >= 64
    }) else {
        return Ok(State {
            key: key.into(),
            error: format!("未找到 {name} 控制接口"),
            ..State::default()
        });
    };
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    let seq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis() as u8;
    let mut frame = [0; 64];
    frame[..7].copy_from_slice(&[5, addr, 0, seq, (1 << 6) | 6, 0x12, 2]);
    io.write(&frame, Duration::from_millis(500))?;
    for _ in 0..16 {
        let reply = io.read(i.input_len as usize, Duration::from_millis(200))?;
        if reply.len() >= 7 && reply[0] == 5 && reply[3] == seq && reply[5] == 0xfe {
            return Err(format!("Jabra 耳机未连接或拒绝查询（0x{:02X}）", reply[6]));
        }
        if let Some((percent, charging)) = parse_battery(&reply, addr, seq) {
            return Ok(State {
                key: key.into(),
                present: true,
                fresh: true,
                percent: Some(percent),
                charging: Some(charging),
                wired: matches!(model, Model::Evolve2Flex),
                identity: format!("{name} · 0B0E:{pid:04X}"),
                updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                settings_error: "仅支持只读电量与充电状态；Windows 实机尚未验证。".into(),
                ..State::default()
            });
        }
    }
    Err("Jabra 耳机未返回有效电量".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn battery_frame() {
        assert_eq!(
            parse_battery(&[5, 0, 4, 7, 0xc8, 0xff, 0, 1, 74], 4, 7),
            Some((74, true))
        );
        assert_eq!(
            parse_battery(&[5, 0, 4, 8, 0xc8, 0xff, 0, 1, 74], 4, 7),
            None
        );
    }
}
