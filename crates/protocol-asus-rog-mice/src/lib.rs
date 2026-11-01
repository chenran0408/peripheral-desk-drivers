//! Read-only ROG/TUF 12 07 battery command from HaloBattery/providers/asus.py (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Spec {
    pub key: &'static str,
    pub name: &'static str,
    pub wireless_pid: u16,
    pub wired_pid: Option<u16>,
    pub steps: bool,
}

pub fn parse(reply: &[u8], steps: bool) -> Option<(u8, bool)> {
    let at = if reply.starts_with(&[0x12, 0x07]) {
        0
    } else if reply.starts_with(&[0, 0x12, 0x07]) {
        1
    } else {
        return None;
    };
    let charging = *reply.get(at + 9)? != 0;
    let raw = reply[at + 4];
    if raw == 0 && !charging {
        return None;
    }
    let percent = if steps {
        if raw > 4 {
            return None;
        }
        raw * 25
    } else {
        if raw > 100 {
            return None;
        }
        raw
    };
    Some((percent, charging))
}

fn candidate(i: &hid::Interface, spec: &Spec) -> bool {
    i.vid == 0x0b05
        && (i.pid == spec.wireless_pid || spec.wired_pid == Some(i.pid))
        && i.path.to_ascii_lowercase().contains("mi_00")
        && i.usage_page >= 0xff00
        && i.input_len >= 64
        && i.output_len >= 64
}

pub fn read_state(spec: &Spec) -> Result<State, String> {
    for i in hid::enumerate()?.iter().filter(|i| candidate(i, spec)) {
        let mut io = WinTransport::open(&i.path)?;
        io.flush()?;
        let mut request = vec![0u8; i.output_len as usize];
        request[1..3].copy_from_slice(&[0x12, 0x07]);
        io.write(&request, Duration::from_millis(300))?;
        let until = Instant::now() + Duration::from_millis(1200);
        while Instant::now() < until {
            let reply = io.read(i.input_len as usize, Duration::from_millis(150))?;
            if reply.starts_with(&[0xff, 0xaa]) || reply.starts_with(&[0, 0xff, 0xaa]) {
                break;
            }
            if let Some((percent, charging)) = parse(&reply, spec.steps) {
                return Ok(State {
                    key: spec.key.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    wired: spec.wired_pid == Some(i.pid),
                    identity: format!("{} · 0B05:{:04X}", spec.name, i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: "仅支持 2.4 GHz/USB 只读电量；未经本项目实机验证。".into(),
                    ..State::default()
                });
            }
        }
    }
    Ok(State {
        key: spec.key.into(),
        error: format!("{} 未连接或处于待机", spec.name),
        ..State::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reply() {
        let mut packet = [0; 65];
        packet[1..3].copy_from_slice(&[0x12, 0x07]);
        packet[5] = 3;
        packet[10] = 1;
        assert_eq!(parse(&packet, true), Some((75, true)));
        packet[5] = 0;
        packet[10] = 0;
        assert_eq!(parse(&packet, true), None);
        packet[1..3].copy_from_slice(&[0xff, 0xaa]);
        assert_eq!(parse(&packet, true), None);
    }
}
