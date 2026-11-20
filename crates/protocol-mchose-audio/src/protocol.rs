use super::transport::Channel;
use super::{field, options, toggle, AudioPage, Family, Model, Result};
use serde_json::json;

fn mc_packet(write: bool, cmd: u8, body: &[u8]) -> Vec<u8> {
    let mut p = vec![
        0xaa,
        1,
        0,
        (body.len() + 1) as u8,
        0,
        if write { 2 } else { 1 },
        cmd,
    ];
    p.extend(body);
    p.push(0);
    p
}
fn mc_body(report: &[u8], cmd: u8) -> Option<&[u8]> {
    if report.len() < 8 || report[..2] != [0xaa, 1] || report[4..7] != [0, 1, cmd] || report[3] == 0
    {
        return None;
    }
    let end = 6 + report[3] as usize;
    let checksum = *report.get(end)?;
    let expected = if report[2] == 1 {
        report[2..end].iter().fold(0, |a, b| a ^ b)
    } else {
        0
    };
    (checksum == expected).then_some(&report[7..end])
}
pub(super) fn mc_read(
    ch: &mut Channel,
    cmd: u8,
    arg: &[u8],
    prefix: &[u8],
    min: usize,
) -> Result<Vec<u8>> {
    ch.query(&mc_packet(false, cmd, arg), |report| {
        let b = mc_body(report, cmd)?;
        (b.starts_with(prefix) && b.len() >= min).then(|| b.to_vec())
    })
}
pub(super) fn mc_write(ch: &mut Channel, cmd: u8, params: &[u8]) -> Result<()> {
    ch.send(&mc_packet(true, cmd, params))
}
const HEADSET: &[(i32, &str)] = &[(0, "游戏一"), (1, "游戏二"), (2, "音乐")];
const SPEAKER: &[(i32, &str)] = &[(0, "音乐"), (1, "游戏"), (2, "电影")];
const EQ: &[(i32, &str)] = &[(0, "游戏一"), (1, "游戏二"), (2, "游戏三")];
pub(super) fn plain(ch: &mut Channel, cmd: &[u8], prefix: u8, min: usize) -> Result<Vec<u8>> {
    ch.query(cmd, |r| {
        (r.len() >= min && r.first() == Some(&prefix)).then(|| r.to_vec())
    })
}
pub fn mode_page(ch: &mut Channel, model: Model, available: bool) -> Result<AudioPage> {
    let mut fields = Vec::new();
    let val = match model.family {
        Family::Oy => {
            let b = plain(ch, &[17], 17, 3)?;
            fields.push(toggle("voice", "语音提示", i32::from(b[1] & 64 != 0)));
            b[2]
        }
        Family::McSpeaker => {
            let b = mc_read(ch, 0, &[], &[], 14)?;
            fields.push(toggle("voice", "语音提示", i32::from(b[0] & 64 != 0)));
            b[1]
        }
        Family::McHeadset => mc_read(ch, 12, &[], &[], 1)?[0],
        _ if available => plain(ch, &[17], 17, 2)?[1],
        _ => 0,
    };
    if available {
        let labels = if model.speaker() {
            SPEAKER
        } else if model.family == Family::McHeadset {
            EQ
        } else {
            HEADSET
        };
        if val <= 2 {
            fields.insert(0, options("mode", "板载音效模式", i32::from(val), labels))
        }
    }
    let note = if !available {
        "固件版本未达到此型号的板载音效模式要求。"
    } else {
        "音效模式保存到所选设备。"
    };
    Ok(AudioPage {
        fields,
        note: note.into(),
        ..Default::default()
    })
}
pub fn mode_write(ch: &mut Channel, model: Model, id: &str, value: i32) -> Result<()> {
    match (model.family, id) {
        (Family::McHeadset | Family::McSpeaker, "mode") => mc_write(ch, 1, &[value as u8]),
        (Family::McSpeaker, "voice") => mc_write(ch, 2, &[value as u8]),
        (Family::Oy, "voice") => ch.send(&[33, 2, value as u8]),
        (_, "mode") => ch.send(&[33, 1, value as u8]),
        _ => Err("所选设备未公开此音效控制".into()),
    }
}
fn turbo_get(ch: &mut Channel, sub: u8) -> Result<u8> {
    ch.query(&[0x22, sub], |r| {
        // M HUB matches the 0x22 reply and decodes byte 2. Queries are serialized and the input queue is flushed before each request.
        if r.len() < 3 || r[0] != 0x22 {
            return None;
        }
        Some(r[2])
    })
}
fn turbo_sleep(raw: u8) -> Option<i32> {
    match raw {
        1 => Some(15),
        4 => Some(30),
        0 => Some(45),
        _ => None,
    }
}
pub fn power_page(ch: &mut Channel, model: Model) -> Result<AudioPage> {
    let (prompt, minutes) = if model.family == Family::McHeadset {
        let a = mc_read(ch, 13, &[], &[], 1)?;
        let b = mc_read(ch, 14, &[], &[], 2)?;
        (a[0], i32::from(u16::from_be_bytes([b[0], b[1]])))
    } else {
        let prompt = turbo_get(ch, 4)?;
        let delay = turbo_get(ch, 2)?;
        (prompt, turbo_sleep(delay).ok_or("设备返回未知关机时间")?)
    };
    if !(1..=16).contains(&prompt) {
        return Err("设备返回未知提示音量".into());
    }
    let delay = if model.family == Family::McHeadset {
        field("minutes", "自动关机（0 为从不）", minutes, 0, 65535, "分钟")
    } else {
        options(
            "minutes",
            "自动关机",
            minutes,
            &[(15, "15 分钟"), (30, "30 分钟"), (45, "45 分钟")],
        )
    };
    Ok(AudioPage {
        fields: vec![
            field("prompt", "提示音量", i32::from(prompt), 1, 16, ""),
            delay,
        ],
        ..Default::default()
    })
}
pub fn power_write(ch: &mut Channel, model: Model, id: &str, value: i32) -> Result<()> {
    match (model.family, id) {
        (Family::McHeadset, "prompt") => mc_write(ch, 33, &[value as u8]),
        (Family::McHeadset, "minutes") => mc_write(ch, 26, &(value as u16).to_be_bytes()),
        (Family::Turbo, "prompt") => ch.send(&[34, 3, value as u8]),
        (Family::Turbo, "minutes") => ch.send(&[
            34,
            0,
            match value {
                15 => 1,
                30 => 4,
                45 => 0,
                _ => return Err("无效关机时间".into()),
            },
        ]),
        _ => Err("未知设备设置".into()),
    }
}
fn parse_bulk<const N: usize>(body: &[u8], mode: u8, kind: u8) -> Option<[u16; N]> {
    if !body.starts_with(&[mode, kind, N as u8]) {
        return None;
    }
    let data = match body.get(3..)? {
        d if d.len() == 2 * N => d,
        [packet, d @ ..] if d.len() == 2 * N && matches!(packet, 0 | 0x10) => d,
        _ => return None,
    };
    Some(std::array::from_fn(|i| {
        u16::from_be_bytes([data[2 * i], data[2 * i + 1]])
    }))
}
pub fn gains(ch: &mut Channel, mode: u8) -> Result<[i32; 10]> {
    let g = mc_read(ch, 4, &[mode, 253], &[mode, 253], 23)?;
    parse_bulk::<10>(&g, mode, 253)
        .map(|v| v.map(|n| i32::from(n as i16)))
        .filter(|a| a.iter().all(|x| (-120..=120).contains(x)))
        .ok_or_else(|| "设备没有返回十段有效增益".into())
}
pub fn write_gains(ch: &mut Channel, mode: u8, gains: &[i32; 10]) -> Result<()> {
    let mut data = vec![mode, 10];
    for g in gains {
        data.extend((*g as i16).to_be_bytes())
    }
    mc_write(ch, 18, &data)
}
pub fn active_eq(ch: &mut Channel) -> Result<u8> {
    let n = mc_read(ch, 12, &[], &[], 1)?[0];
    if n > 2 {
        return Err("设备返回未知板载 EQ 槽".into());
    }
    Ok(n)
}
pub fn read_name(ch: &mut Channel, mode: u8) -> Result<String> {
    let b = mc_read(ch, 5, &[mode], &[mode], 1)?;
    let name = &b[1..];
    let end = name.iter().position(|v| *v == 0).unwrap_or(name.len());
    String::from_utf8(name[..end].to_vec()).map_err(|_| "板载配置名格式未知".into())
}
pub fn write_name(ch: &mut Channel, mode: u8, name: &str) -> Result<()> {
    if name.len() > 54 || name.contains('\0') {
        return Err("板载配置名最多 54 个 UTF-8 字节".into());
    }
    let mut bytes = vec![mode];
    bytes.extend(name.as_bytes());
    mc_write(ch, 24, &bytes)
}
pub fn eq_page(ch: &mut Channel, mode: u8, selector: Option<i32>) -> Result<AudioPage> {
    let i = usize::try_from(selector.unwrap_or(0))
        .ok()
        .filter(|i| *i < 10)
        .ok_or("无效 EQ 频段")?;
    let gain = gains(ch, mode)?;
    let frequencies = mc_read(ch, 4, &[mode, 254], &[mode, 254], 23)
        .ok()
        .and_then(|b| parse_bulk::<10>(&b, mode, 254));
    let titles: Vec<String> = (0..10)
        .map(|n| frequencies.map_or_else(|| format!("频段 {}", n + 1), |f| format!("{} Hz", f[n])))
        .collect();
    let mut gain_field = field("gain", "频段增益", gain[i], -120, 120, "dB");
    gain_field["scale"] = json!(10);
    gain_field["step"] = json!(1);
    Ok(AudioPage {
        fields: vec![gain_field],
        selector: Some(
            json!({"label":"频段","value":i,"choices":titles.iter().enumerate().map(|(n,t)|super::choice(n as i32,t)).collect::<Vec<_>>()}),
        ),
        curve: gain
            .iter()
            .enumerate()
            .map(|(n, g)| json!({"value":n,"label":titles[n],"db":f64::from(*g)/10.0}))
            .collect(),
        note: read_name(ch, mode)
            .ok()
            .filter(|n| !n.is_empty())
            .map_or_else(
                || "编辑所选板载槽；在「音效模式」切换当前槽。".into(),
                |name| format!("板载配置 · {name}"),
            ),
    })
}
pub fn state(model: Model, ch: &mut Channel) -> Result<(Option<u8>, Option<bool>)> {
    match model.family {
        Family::McHeadset => {
            let b = mc_read(ch, 11, &[], &[], 2)?;
            let p = b[1] & 127;
            Ok(if b[0] == 0 || p > 100 {
                (None, None)
            } else {
                (Some(p), Some(b[1] & 128 != 0))
            })
        }
        Family::Blk | Family::Turbo => {
            let b = ch.query(&[101, 1], |r| {
                (r.len() >= 3 && r[0] == 101 && r[2] != 0).then(|| r.to_vec())
            })?;
            Ok(if b[2] == 26 || b[1] > 100 {
                (None, None)
            } else {
                (
                    Some(b[1]),
                    match b[2] {
                        3 | 4 => Some(true),
                        2 => Some(false),
                        _ => None,
                    },
                )
            })
        }
        _ => Ok((None, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc_checks_length_type_checksum_and_selector() {
        let mut p = mc_packet(false, 4, &[0, 253, 1, 0xff, 0xce]);
        assert_eq!(mc_body(&p, 4), Some(&[0, 253, 1, 0xff, 0xce][..]));
        assert_eq!(mc_body(&p, 5), None);
        p[2] = 1;
        assert!(mc_body(&p, 4).is_none());
        let end = p.len() - 1;
        p[end] = p[2..end].iter().fold(0, |a, b| a ^ b);
        assert!(mc_body(&p, 4).is_some());
        assert_eq!(
            parse_bulk::<1>(&[0, 253, 1, 0x10, 0xff, 0xce], 0, 253),
            Some([(-50_i16) as u16])
        );
        assert!(parse_bulk::<1>(&[1, 253, 1, 0x10, 0xff, 0xce], 0, 253).is_none());
        assert_eq!(turbo_sleep(0), Some(45));
        assert_eq!(turbo_sleep(1), Some(15));
    }
}
