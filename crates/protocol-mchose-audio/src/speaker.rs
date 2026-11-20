use super::{field, options, toggle, AudioPage, Family, Model, Result};
use super::{
    protocol::{mc_read, mc_write, plain},
    transport::Channel,
};
use std::collections::BTreeMap;
type Changes = BTreeMap<String, i32>;
struct Light {
    mode: u8,
    bright: u8,
    cycle: u8,
    color: [u8; 3],
    smart: bool,
    groups: u8,
}
fn lighting(ch: &mut Channel, model: Model) -> Result<Light> {
    let mc = model.family == Family::McSpeaker;
    let b = if mc {
        mc_read(ch, 0, &[], &[], 14)?
    } else {
        plain(ch, &[17], 17, 15)?
    };
    let i = if mc { 2 } else { 3 };
    let c = if mc { 11 } else { 12 };
    Ok(Light {
        mode: b[i],
        bright: b[i + 1],
        cycle: b[i + 2],
        color: [b[c], b[c + 1], b[c + 2]],
        smart: mc && b[0] & 128 != 0,
        groups: b[i + 7],
    })
}
fn rgb_fields(color: [u8; 3]) -> Vec<serde_json::Value> {
    ["red", "green", "blue"]
        .into_iter()
        .zip(["红", "绿", "蓝"])
        .zip(color)
        .map(|((id, label), v)| field(id, label, v.into(), 0, 255, ""))
        .collect()
}
fn value(values: &Changes, k: &str, old: u8) -> u8 {
    values.get(k).copied().unwrap_or(i32::from(old)) as u8
}
fn color(values: &Changes, old: [u8; 3]) -> [u8; 3] {
    [
        value(values, "red", old[0]),
        value(values, "green", old[1]),
        value(values, "blue", old[2]),
    ]
}
pub(super) fn lights_page(ch: &mut Channel, model: Model) -> Result<AudioPage> {
    let s = lighting(ch, model)?;
    if s.mode > 7 {
        return Err("设备返回未知灯效模式".into());
    }
    let mut fields = vec![field(
        "mode",
        "灯效模式（0 为关闭）",
        s.mode.into(),
        0,
        7,
        "",
    )];
    if s.mode > 0 {
        fields.push(field(
            "brightness",
            "亮度",
            s.bright.into(),
            i32::from(model.id == "mchose-k20" && s.mode == 5),
            100,
            "%",
        ));
        fields.push(field("cycle", "灯效周期", s.cycle.into(), 0, 255, ""));
        fields.extend(rgb_fields(s.color));
    }
    if model.family == Family::McSpeaker {
        fields.push(toggle("smart", "智能灯光", i32::from(s.smart)))
    } else if (1..=3).contains(&s.groups) {
        fields.push(options(
            "groups",
            "灯组",
            s.groups.into(),
            &[(1, "第一组"), (2, "第二组"), (3, "两组")],
        ))
    }
    Ok(AudioPage {
        fields,
        ..Default::default()
    })
}
pub(super) fn lights_write(ch: &mut Channel, model: Model, v: &Changes) -> Result<()> {
    let s = lighting(ch, model)?;
    let mc = model.family == Family::McSpeaker;
    if let Some(&m) = v.get("mode") {
        return if mc {
            mc_write(ch, 7, &[m as u8])
        } else {
            ch.send(&[33, 6, m as u8])
        };
    }
    if v.contains_key("brightness") || v.contains_key("cycle") {
        let detail = [
            s.mode,
            value(v, "brightness", s.bright),
            value(v, "cycle", s.cycle),
            0,
        ];
        if mc {
            mc_write(ch, 3, &detail)?
        } else {
            let mut args = vec![33, 3];
            args.extend(detail);
            ch.send(&args)?
        }
    }
    if ["red", "green", "blue"].iter().any(|k| v.contains_key(*k)) {
        let [r, g, b] = color(v, s.color);
        if mc {
            mc_write(ch, 14, &[0, 1, g, r, b])?
        } else {
            ch.send(&[34, 0, 1, g, r, b])?
        }
    }
    if let Some(&x) = v.get("smart") {
        mc_write(ch, 6, &[x as u8])?
    }
    if let Some(&x) = v.get("groups") {
        ch.send(&[33, 5, 3, x as u8])?
    }
    Ok(())
}
struct Screen {
    on: bool,
    color: [u8; 3],
    mode: u8,
    theme: u8,
    indices: [u8; 9],
}
fn screen(ch: &mut Channel) -> Result<Screen> {
    let b = mc_read(ch, 1, &[], &[], 24)?;
    let mut indices = [0; 9];
    indices[..8].copy_from_slice(&b[14..22]);
    indices[8] = b[23];
    Ok(Screen {
        on: b[0] != 0,
        color: [b[2], b[1], b[3]],
        mode: b[4],
        theme: b[5],
        indices,
    })
}
pub(super) fn screen_page(ch: &mut Channel) -> Result<AudioPage> {
    let s = screen(ch)?;
    let mut fields = Vec::new();
    if s.theme < 9 {
        fields.push(toggle("screen", "屏幕开关", i32::from(s.on)));
        fields.push(field("theme", "屏幕主题", s.theme.into(), 0, 8, ""));
    }
    fields.extend(rgb_fields(s.color));
    Ok(AudioPage {
        fields,
        ..Default::default()
    })
}
pub(super) fn screen_write(ch: &mut Channel, v: &Changes) -> Result<()> {
    let s = screen(ch)?;
    let [r, g, b] = color(v, s.color);
    if v.contains_key("screen") || v.contains_key("theme") {
        let theme = value(v, "theme", s.theme);
        let index = s
            .indices
            .get(usize::from(theme))
            .ok_or("设备返回未知屏幕主题")?;
        mc_write(
            ch,
            9,
            &[
                value(v, "screen", u8::from(s.on)),
                g,
                r,
                b,
                s.mode,
                theme,
                *index,
            ],
        )
    } else {
        mc_write(ch, 31, &[g, r, b])
    }
}

// A lighting mode owns its own color/brightness settings. Snapshot the destination
// bank after switching, so a failed combined edit restores both banks.
pub(super) fn switch_with_edits(
    ch: &mut Channel,
    model: Model,
    old_mode: i32,
    mode: i32,
    mut edits: Changes,
) -> Result<AudioPage> {
    let set = |ch: &mut Channel, values: &Changes| -> Result<AudioPage> {
        lights_write(ch, model, values)?;
        let page = lights_page(ch, model)?;
        super::confirmed(&page, values)?;
        Ok(page)
    };
    let mut undo = Changes::new();
    edits.remove("mode");
    let result = (|| {
        let dest = set(ch, &Changes::from([("mode".into(), mode)]))?;
        let (before, after) = super::changes(&dest, &serde_json::json!(edits))?;
        undo = before;
        if after.is_empty() {
            Ok(dest)
        } else {
            set(ch, &after)
        }
    })();
    match result {
        Ok(page) => Ok(page),
        Err(error) => {
            let mut failures = Vec::new();
            if !undo.is_empty() {
                if let Err(e) = set(ch, &undo) {
                    failures.push(e)
                }
            }
            if let Err(e) = set(ch, &Changes::from([("mode".into(), old_mode)])) {
                failures.push(e)
            }
            if failures.is_empty() {
                Err(format!("{error}；灯效模式与涉及的设置已恢复并读回"))
            } else {
                Err(format!("{error}；恢复未确认：{}", failures.join("、")))
            }
        }
    }
}
