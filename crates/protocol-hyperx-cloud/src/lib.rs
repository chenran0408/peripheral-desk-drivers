//! HyperX exchanges from HaloBattery providers/hyperx*.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const ALPHA_KEY: &str = "hyperx-cloud-alpha-2";
pub const CORE_KEY: &str = "hyperx-cloud-ii-core-wireless";
pub const II_KEY: &str = "hyperx-cloud-ii-wireless";
pub const IIIS_KEY: &str = "hyperx-cloud-iii-s-wireless";
pub const III_KEY: &str = "hyperx-cloud-iii-wireless";

#[derive(Clone, Copy)]
pub enum Model {
    Alpha,
    Core,
    Ii,
    IiiS,
    Iii,
}

impl Model {
    fn key(self) -> &'static str {
        match self {
            Self::Alpha => ALPHA_KEY,
            Self::Core => CORE_KEY,
            Self::Ii => II_KEY,
            Self::IiiS => IIIS_KEY,
            Self::Iii => III_KEY,
        }
    }
    fn accepts(self, i: &hid::Interface) -> bool {
        if i.vid != 0x03f0 {
            return false;
        }
        match self {
            Self::Alpha => i.pid == 0x08be && (i.usage_page, i.usage) == (0xff13, 0xff00),
            Self::Core => {
                matches!(i.pid, 0x0995 | 0x0795) && (i.usage_page, i.usage) == (0xff13, 1)
            }
            Self::Ii => {
                matches!(i.pid, 0x0696 | 0x018b) && (i.usage_page, i.usage) == (0xff90, 0x0303)
            }
            Self::IiiS => matches!(i.pid, 0x02cc | 0x06be),
            Self::Iii => matches!(i.pid, 0x05b7 | 0x0c9d) && (i.usage_page, i.usage) == (0xff13, 1),
        }
    }
}

pub fn request(model: Model, cmd: u8) -> Vec<u8> {
    let (len, prefix): (usize, &[u8]) = match model {
        Model::Alpha => (64, &[0x50, 2]),
        Model::Core | Model::Iii => (62, &[0x66, cmd]),
        Model::Ii => (52, &[6, 0xff, 0xbb, cmd, 0]),
        Model::IiiS => (64, &[0x0c, 2, 3, 1, 0, cmd]),
    };
    let mut out = vec![0; len];
    out[..prefix.len()].copy_from_slice(prefix);
    out
}

pub fn parse(model: Model, cmd: u8, reply: &[u8]) -> Option<u8> {
    match model {
        Model::Alpha
            if cmd == 2 && reply.len() > 6 && reply[..2] == [0x51, 2] && reply[2] <= 100 =>
        {
            Some(reply[2])
        }
        Model::Alpha => None,
        Model::Core | Model::Iii if reply.len() >= 3 && reply[0] == 0x66 => match cmd {
            0x89 if reply.len() >= 5
                && matches!(reply[1], 0x89 | 0x0d)
                && (reply[2] != 0 || reply[3] != 0)
                && reply[4] <= 100 =>
            {
                Some(reply[4])
            }
            0x8a if matches!(reply[1], 0x8a | 0x0c) && reply[2] <= 2 => Some(reply[2]),
            _ => None,
        },
        Model::Ii if reply.len() > 7 && reply[..4] == [6, 0xff, 0xbb, cmd] => match cmd {
            2 if reply[7] <= 100 => Some(reply[7]),
            3 => Some(u8::from(reply[4] == 1)),
            _ => None,
        },
        Model::IiiS
            if reply.len() >= 7 && reply[0] == 0x0c && reply[5] == cmd && reply[6] != 0xff =>
        {
            Some(reply[6])
        }
        Model::IiiS
            if reply.len() >= 6
                && reply[0] == 0x0d
                && reply[4] == if cmd == 6 { 1 } else { 10 } =>
        {
            Some(reply[5])
        }
        _ => None,
    }
}

pub fn parse_alpha(reply: &[u8]) -> Option<(u8, bool)> {
    parse(Model::Alpha, 2, reply).map(|percent| (percent, reply[6] & 0x80 != 0))
}

fn state(model: Model, i: &hid::Interface, percent: u8, charging: Option<bool>) -> State {
    State {
        key: model.key().into(),
        present: true,
        fresh: true,
        percent: Some(percent),
        charging,
        identity: format!("HyperX · 03F0:{:04X}", i.pid),
        updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        settings_error: "仅支持只读电量与充电状态。".into(),
        ..State::default()
    }
}

fn query(io: &mut WinTransport, model: Model, cmd: u8, path: &str) -> Result<Option<u8>, String> {
    let data = request(model, cmd);
    if let Err(e) = io.write(&data, Duration::from_millis(500)) {
        if matches!(model, Model::Iii) {
            hid::Device::open(path)?.send(&data)?;
        } else {
            return Err(e);
        }
    }
    let end = Instant::now() + Duration::from_millis(1500);
    while Instant::now() < end {
        let input_len = match model {
            Model::Ii => 20,
            Model::Core | Model::Iii => 62,
            Model::Alpha | Model::IiiS => 64,
        };
        let reply = io.read(input_len, Duration::from_millis(150))?;
        if let Some(value) = parse(model, cmd, &reply) {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn read_iii_s(interfaces: &[hid::Interface]) -> Result<State, String> {
    for pid in [0x02cc, 0x06be] {
        let mut handles: Vec<_> = interfaces
            .iter()
            .filter(|i| i.pid == pid)
            .filter_map(|i| WinTransport::open(&i.path).ok().map(|io| (i, io)))
            .collect();
        handles.sort_by_key(|(i, _)| i.usage_page < 0xff00);
        if handles.is_empty() {
            continue;
        }
        let mut ask = |cmd| -> Option<u8> {
            let data = request(Model::IiiS, cmd);
            if !handles
                .iter_mut()
                .any(|(_, io)| io.write(&data, Duration::from_millis(500)).is_ok())
            {
                return None;
            }
            let end = Instant::now() + Duration::from_secs(1);
            while Instant::now() < end {
                for (_, io) in handles.iter_mut() {
                    if let Ok(reply) = io.read(64, Duration::from_millis(10)) {
                        if let Some(value) = parse(Model::IiiS, cmd, &reply) {
                            return Some(value);
                        }
                    }
                }
            }
            None
        };
        if let Some(percent) = ask(6).filter(|v| *v <= 100) {
            let charging = ask(0x48).and_then(|v| match v {
                0 => Some(false),
                1 | 2 => Some(true),
                _ => None,
            });
            return Ok(state(Model::IiiS, handles[0].0, percent, charging));
        }
    }
    Err("Cloud III S 未返回有效电量".into())
}

pub fn read_state(model: Model) -> Result<State, String> {
    let mut interfaces: Vec<_> = hid::enumerate()?
        .into_iter()
        .filter(|i| model.accepts(i))
        .collect();
    if interfaces.is_empty() {
        return Ok(State {
            key: model.key().into(),
            error: "未找到 HyperX 电量控制接口".into(),
            ..State::default()
        });
    }
    if matches!(model, Model::IiiS) {
        return read_iii_s(&interfaces);
    }
    if matches!(model, Model::Core) {
        interfaces.sort_by_key(|i| i.pid != 0x0795);
    }
    let mut best = None;
    for i in &interfaces {
        let Ok(mut io) = WinTransport::open(&i.path) else {
            continue;
        };
        if matches!(model, Model::Alpha) {
            for _ in 0..64 {
                if io.read(64, Duration::from_millis(5))?.is_empty() {
                    break;
                }
            }
            io.write(&request(model, 2), Duration::from_millis(500))?;
            let end = Instant::now() + Duration::from_millis(1500);
            while Instant::now() < end {
                if let Some((percent, charging)) =
                    parse_alpha(&io.read(64, Duration::from_millis(150))?)
                {
                    return Ok(state(model, i, percent, Some(charging)));
                }
            }
            continue;
        } else {
            io.flush()?;
        }
        let level_cmd = if matches!(model, Model::Alpha | Model::Ii) {
            2
        } else if matches!(model, Model::Core | Model::Iii) {
            0x89
        } else {
            6
        };
        let Some(percent) = query(&mut io, model, level_cmd, &i.path)? else {
            continue;
        };
        let charging = match model {
            Model::Alpha => unreachable!(),
            Model::Ii => query(&mut io, model, 3, &i.path)?.map(|v| v == 1),
            _ => query(&mut io, model, 0x8a, &i.path)?.and_then(|v| match v {
                0 => Some(false),
                1 | 2 => Some(true),
                _ => None,
            }),
        };
        let candidate = state(model, i, percent, charging);
        if charging == Some(true) {
            return Ok(candidate);
        }
        if best.is_none() {
            best = Some(candidate);
        }
    }
    best.ok_or_else(|| "HyperX 未返回有效电量".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames() {
        assert_eq!(&request(Model::Alpha, 2)[..3], &[0x50, 2, 0]);
        assert_eq!(
            parse(Model::Alpha, 2, &[0x51, 2, 67, 0, 0, 0, 0x80]),
            Some(67)
        );
        assert_eq!(&request(Model::IiiS, 6)[..6], &[12, 2, 3, 1, 0, 6]);
        assert_eq!(
            parse(Model::Ii, 2, &[6, 255, 187, 2, 0, 0, 0, 55]),
            Some(55)
        );
        assert_eq!(
            parse(Model::Core, 0x89, &[0x66, 0x89, 0x0e, 0xd7, 48]),
            Some(48)
        );
        assert_eq!(parse(Model::Core, 0x8a, &[0x66, 0x8a, 0, 0]), Some(0));
    }
}
