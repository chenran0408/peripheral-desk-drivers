//! Battery exchanges from HeadsetControl SteelSeries protocol facts (GPL reference).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Arctis7,
    Arctis9,
    ProWireless,
    Arctis7Plus,
    GameBuds,
    Arctis1,
}

impl Model {
    pub fn key(self) -> &'static str {
        match self {
            Self::Arctis7 => "steelseries-arctis-7-2019",
            Self::Arctis9 => "steelseries-arctis-9",
            Self::ProWireless => "steelseries-arctis-pro-wireless",
            Self::Arctis7Plus => "steelseries-arctis-7-plus",
            Self::GameBuds => "steelseries-gamebuds",
            Self::Arctis1 => "steelseries-arctis-1",
        }
    }
    pub fn pids(self) -> &'static [u16] {
        match self {
            Self::Arctis7 => &[0x12ad],
            Self::Arctis9 => &[0x12c2, 0x12c4],
            Self::ProWireless => &[0x1290],
            Self::Arctis7Plus => &[0x220e, 0x2212, 0x2216, 0x2236],
            Self::GameBuds => &[0x230a],
            Self::Arctis1 => &[0x12b3, 0x12b6],
        }
    }
    fn request(self, len: usize) -> Vec<u8> {
        let mut frame = vec![0; len];
        let command: &[u8] = match self {
            Self::Arctis7 => &[0x06, 0x18],
            Self::Arctis1 => &[0x06, 0x12],
            Self::Arctis9 => &[0x00, 0x20],
            Self::ProWireless => &[0x41, 0xaa],
            Self::Arctis7Plus | Self::GameBuds => &[0x00, 0xb0],
        };
        frame[..command.len()].copy_from_slice(command);
        frame
    }
}

pub fn parse(model: Model, reply: &[u8]) -> Option<(u8, bool)> {
    match model {
        Model::Arctis7 => {
            if reply.len() < 3 || reply[..2] != [6, 0x18] || !(1..=100).contains(&reply[2]) {
                return None;
            }
            Some((reply[2], false))
        }
        Model::Arctis1 => {
            if reply.len() < 4 || reply[..2] != [6, 0x12] || reply[2] == 1 || reply[3] > 100 {
                return None;
            }
            Some((reply[3], false))
        }
        Model::Arctis9 => {
            if reply.len() < 5 || reply[..2] != [0xaa, 1] || !(0x64..=0x9a).contains(&reply[3]) {
                return None;
            }
            Some((((reply[3] - 0x64) as u16 * 100 / 0x36) as u8, reply[4] == 1))
        }
        Model::ProWireless => (reply.first().copied()? <= 4).then_some((reply[0] * 25, false)),
        Model::Arctis7Plus => {
            let r = reply.strip_prefix(&[0]).unwrap_or(reply);
            if r.len() < 4 || r[0] != 0xb0 || r[1] == 1 || r[2] > 4 {
                return None;
            }
            Some((r[2] * 25, r[3] == 1))
        }
        Model::GameBuds => {
            let r = reply.strip_prefix(&[0]).unwrap_or(reply);
            if r.len() < 7 || r[0] != 0xb0 {
                return None;
            }
            let left = (r[3] == 3 && r[5] <= 100).then_some(r[5]);
            let right = (r[4] == 3 && r[6] <= 100).then_some(r[6]);
            match (left, right) {
                (Some(a), Some(b)) => Some((a.min(b), false)),
                (Some(a), None) | (None, Some(a)) => Some((a, false)),
                _ => None,
            }
        }
    }
}

fn candidate(i: &hid::Interface, model: Model) -> bool {
    if i.vid != 0x1038 || !model.pids().contains(&i.pid) || i.input_len == 0 || i.output_len < 2 {
        return false;
    }
    let path = i.path.to_ascii_lowercase();
    match model {
        Model::Arctis7 => i.usage_page == 0xff43 && i.usage == 0x0202 && path.contains("mi_05"),
        Model::Arctis1 => i.usage_page == 0xff43 && i.usage == 0x0202 && path.contains("mi_03"),
        Model::Arctis9 | Model::ProWireless => path.contains("mi_00"),
        Model::Arctis7Plus | Model::GameBuds => {
            i.usage_page == 0xffc0 && i.usage == 1 && path.contains("mi_03")
        }
    }
}

fn await_reply(
    io: &mut WinTransport,
    len: usize,
    model: Model,
) -> Result<Option<(u8, bool)>, String> {
    let end = Instant::now() + Duration::from_millis(1000);
    while Instant::now() < end {
        if let Some(value) = parse(model, &io.read(len, Duration::from_millis(100))?) {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

pub fn read_state(model: Model) -> Result<State, String> {
    let interfaces = hid::enumerate()?;
    let iface = interfaces
        .iter()
        .find(|i| candidate(i, model))
        .ok_or("未找到 Arctis 指定电量接口")?;
    let mut io = WinTransport::open(&iface.path)?;
    io.flush()?;
    if model == Model::ProWireless {
        io.write(
            &model.request(iface.output_len as usize),
            Duration::from_millis(500),
        )?;
        let end = Instant::now() + Duration::from_millis(500);
        let mut online = false;
        while Instant::now() < end {
            match io
                .read(iface.input_len as usize, Duration::from_millis(100))?
                .first()
            {
                Some(4) => {
                    online = true;
                    break;
                }
                Some(2) => return Err("Arctis Pro Wireless 耳机未连接".into()),
                _ => {}
            }
        }
        if !online {
            return Err("Arctis Pro Wireless 基站未回复连接状态".into());
        }
        let mut request = vec![0; iface.output_len as usize];
        request[..2].copy_from_slice(&[0x40, 0xaa]);
        io.write(&request, Duration::from_millis(500))?;
    } else {
        io.write(
            &model.request(iface.output_len as usize),
            Duration::from_millis(500),
        )?;
    }
    let (percent, charging) = await_reply(&mut io, iface.input_len as usize, model)?
        .ok_or("Arctis 未回复有效电量（可能关机或超出范围）")?;
    Ok(State {
        key: model.key().into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging: Some(charging),
        identity: format!("SteelSeries Arctis · 1038:{:04X}", iface.pid),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅支持只读电量查询。".into(),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layouts() {
        assert_eq!(parse(Model::Arctis7, &[6, 0x18, 75]), Some((75, false)));
        assert_eq!(
            parse(Model::Arctis1, &[6, 0x12, 0, 75, 0, 0, 0, 0]),
            Some((75, false))
        );
        assert_eq!(
            parse(Model::Arctis9, &[0xaa, 1, 0, 0x9a, 1]),
            Some((100, true))
        );
        assert_eq!(parse(Model::ProWireless, &[3]), Some((75, false)));
        assert_eq!(
            parse(Model::Arctis7Plus, &[0xb0, 3, 4, 1]),
            Some((100, true))
        );
        assert_eq!(
            parse(Model::GameBuds, &[0xb0, 0, 0, 3, 3, 64, 33]),
            Some((33, false))
        );
        assert_eq!(parse(Model::GameBuds, &[0xb0, 0, 0, 2, 2, 0, 0]), None);
    }
}
