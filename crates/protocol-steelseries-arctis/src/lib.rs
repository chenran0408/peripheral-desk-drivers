//! Read-only Arctis status exchange from HeadsetControl and HaloBattery (MIT).
use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const NOVA7_KEY: &str = "steelseries-arctis-nova-7";
pub const PRO_KEY: &str = "steelseries-arctis-nova-pro-wireless";
const LEN: usize = 64;
const DISCRETE: &[u16] = &[0x2202, 0x2206, 0x223a, 0x227a, 0x22a4, 0x220a];
pub const NOVA7: &[u16] = &[
    0x22a1, 0x2202, 0x227e, 0x2206, 0x2258, 0x229e, 0x22ad, 0x22a4, 0x22a5, 0x223a, 0x22a9, 0x227a,
    0x220a, 0x22a7, 0x2298, 0x2232, 0x2253,
];

pub fn request(pro: bool) -> Vec<u8> {
    let mut frame = vec![0; if pro { 31 } else { LEN }];
    frame[..2].copy_from_slice(&[if pro { 6 } else { 0 }, 0xb0]);
    frame
}

pub fn parse(reply: &[u8], pid: u16, pro: bool) -> Option<(u8, bool)> {
    if pro {
        if reply.len() < 16 || !matches!(reply[15], 1 | 2 | 8) || reply[15] == 1 || reply[6] > 8 {
            return None;
        }
        return Some((((reply[6] as u16 * 100) / 8) as u8, reply[15] == 2));
    }
    let r = reply.strip_prefix(&[0]).unwrap_or(reply);
    if matches!(pid, 0x2232 | 0x2253) {
        if r.len() < 5 || r[0] != 0xb0 || r[1] == 2 || r[3] > 100 {
            return None;
        }
        return Some((r[3], r[4] == 1));
    }
    if r.len() < 4 || r[0] != 0xb0 || r[1] != 3 || !matches!(r[3], 1..=3) {
        return None;
    }
    let percent = if DISCRETE.contains(&pid) {
        if r[2] > 4 {
            return None;
        }
        r[2] * 25
    } else {
        if r[2] > 100 {
            return None;
        }
        r[2]
    };
    Some((percent, matches!(r[3], 1 | 2)))
}

fn candidate(i: &hid::Interface, pro: bool) -> bool {
    i.vid == 0x1038
        && (if pro {
            matches!(i.pid, 0x12e0 | 0x12e5)
        } else {
            NOVA7.contains(&i.pid)
        })
        && i.usage_page >= 0xff00
        && (pro || (i.usage_page == 0xffc0 && i.usage == 1))
        && (i.path.to_ascii_lowercase().contains("mi_03")
            || (pro && i.path.to_ascii_lowercase().contains("mi_04")))
        && i.input_len as usize >= 16
        && i.output_len as usize >= if pro { 31 } else { LEN }
}

pub fn read_state(pro: bool) -> Result<State, String> {
    let key = if pro { PRO_KEY } else { NOVA7_KEY };
    let interfaces = hid::enumerate()?;
    let mut choices: Vec<_> = interfaces.iter().filter(|i| candidate(i, pro)).collect();
    choices.sort_by_key(|i| i.usage_page != 0xffc0);
    if choices.is_empty() {
        return Ok(State {
            key: key.into(),
            error: "未找到 Arctis 指定电量接口".into(),
            ..State::default()
        });
    }
    for i in choices {
        let mut transport = WinTransport::open(&i.path)?;
        transport.flush()?;
        transport.write(&request(pro), Duration::from_millis(500))?;
        let end = Instant::now() + Duration::from_millis(1000);
        while Instant::now() < end {
            let reply = transport.read(i.input_len as usize, Duration::from_millis(100))?;
            if let Some((percent, charging)) = parse(&reply, i.pid, pro) {
                return Ok(State {
                    key: key.into(),
                    present: true,
                    fresh: true,
                    percent: Some(percent),
                    charging: Some(charging),
                    identity: format!("SteelSeries Arctis · 1038:{:04X}", i.pid),
                    updated: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    settings_error: if pro {
                        "电量为九档估计值；仅提供只读状态。"
                    } else {
                        "仅提供只读电量与充电状态。"
                    }
                    .into(),
                    ..State::default()
                });
            }
        }
    }
    Err("Arctis 耳机未回复有效电量（可能关机或超出范围）".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layouts() {
        assert_eq!(parse(&[0xb0, 3, 77, 3], 0x22a1, false), Some((77, false)));
        assert_eq!(parse(&[0xb0, 3, 3, 1], 0x2202, false), Some((75, true)));
        assert_eq!(parse(&[0xb0, 1, 0, 65, 1], 0x2232, false), Some((65, true)));
        let mut pro = [0; 64];
        pro[6] = 5;
        pro[15] = 8;
        assert_eq!(parse(&pro, 0x12e0, true), Some((62, false)));
        pro[15] = 3;
        assert_eq!(parse(&pro, 0x12e0, true), None);
    }
}
