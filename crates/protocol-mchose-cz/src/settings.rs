use super::*;
use wire::{commit, Edit, Region};

pub fn fields_performance(m: Model, b: &[u8], info: &[u8]) -> Vec<Value> {
    let mut rates = vec![(4, "1000 Hz")];
    if m.kind != 101 {
        rates.extend([(3, "2000 Hz"), (2, "4000 Hz"), (1, "8000 Hz")]);
    }
    if matches!(m.kind, 135 | 136 | 138 | 211 | 213) && info[6] & 3 == 3 {
        rates.push((0, "16000 Hz"));
    }
    let mut fields = vec![
        choice("rate", "回报率", i32::from(b[4] & 15), options(&rates)),
        bit("bottom", "稳定性模式（防误触）", b[7] & 2 != 0),
        bit("stable", "按键防抖优化", b[7] >> 5 != 0),
        bit("check", "自动校准", b[7] & 4 != 0),
        bit("win", "锁定 Windows 键", b[6] & 1 != 0),
        bit("tachyon", "极限模式", b[7] & 1 != 0),
        bit("mac", "Mac 模式", b[1] & 3 == 2),
    ];
    if m.smart() {
        fields.push(number(
            "smart",
            "智能 RT（0 为关闭）",
            i32::from(b[20]),
            0,
            15,
        ));
    }
    if m.sleep() {
        fields.push(bit("never", "永不休眠", b[36] == 1));
        let mut f = number("sleep", "休眠时间", i32::from(b[35]) * 30, 30, 7650);
        f["step"] = json!(30);
        f["unit"] = json!("秒");
        fields.push(f);
    }
    fields
}
pub fn edit_perf(id: &str, n: i32, b: &mut [u8], slot: usize) {
    let v = n as u8;
    match id {
        "tachyon" => put_bits(&mut b[7], 1, v),
        "mac" => {
            put_bits(&mut b[1], 15, 4 * slot as u8 + 2 * v);
            if v == 1 {
                put_bits(&mut b[6], 1, 0)
            }
        }
        "rate" => put_bits(&mut b[4], 15, v),
        "bottom" => put_bits(&mut b[7], 2, v << 1),
        "stable" => put_bits(&mut b[7], 0xe0, v << 5),
        "win" => put_bits(&mut b[6], 1, v),
        "check" => put_bits(&mut b[7], 4, v << 2),
        "smart" => b[20] = v,
        "sleep" => {
            b[35] = (n / 30) as u8;
            b[36] = 0
        }
        "never" => b[36] = v,
        _ => {}
    }
}
fn val9(b: &[u8], i: usize) -> i32 {
    1 + i32::from(u16::from_le_bytes([b[i], b[i + 1] & 1]))
}
fn set9(b: &mut [u8], i: usize, n: i32) {
    let raw = (n - 1) as u16;
    let [lo, hi] = raw.to_le_bytes();
    b[i] = lo;
    put_bits(&mut b[i + 1], 1, hi);
}
fn base(m: Model) -> i32 {
    if m.kind == 101 {
        50
    } else {
        100
    }
}
fn rt_base(m: Model, precision: u8) -> i32 {
    match precision {
        1 => 200,
        2 => 1000,
        _ => base(m),
    }
}
fn distance(id: &str, label: &str, ticks: i32, lo: i32, hi: i32, base: i32) -> Value {
    let unit = 1000 / base;
    let mut f = number(id, label, ticks * unit, lo * unit, hi * unit);
    f["scale"] = json!(1000);
    f["step"] = json!(unit);
    f["unit"] = json!("mm");
    f
}
pub fn fields_rt(m: Model, b: &[u8], tachyon: bool, fw: u16) -> Vec<Value> {
    let base = base(m);
    let max = i32::from(m.travel_max[usize::from(b[0] & 15)]);
    let max_normal = max * base / 100;
    let p = rt_base(m, b[3] >> 3 & 3);
    let r = rt_base(m, b[3] >> 1 & 3);
    let modes = precision_modes(m, tachyon, fw);
    let finest = rt_base(m, modes[0]);
    let rt_field = |id: &str, label: &str, ticks: i32, actual_base: i32| {
        let mut f = distance(id, label, ticks, 1, 1, actual_base);
        let unit = 1000 / finest;
        f["step"] = json!(unit);
        f["min"] = json!(if tachyon { unit } else { 2 * unit });
        f["max"] = json!(max * 10);
        f
    };
    let mut fields = vec![
        choice(
            "mode",
            "触发方式",
            i32::from(b[1] & 15),
            options(&[(0, "定点触发"), (1, "快速触发 RT"), (2, "连续快速触发")]),
        ),
        distance(
            "act",
            "触发行程",
            val9(b, 2),
            1,
            max_normal - if m.kind == 101 { 4 } else { 9 },
            base,
        ),
        rt_field("press", "RT 按下灵敏度", val9(b, 4), p),
        rt_field("release", "RT 抬起灵敏度", val9(b, 6), r),
        distance(
            "upper",
            "顶部死区",
            i32::from(b[5] >> 1),
            0,
            50.min(max_normal),
            base,
        ),
        distance(
            "lower",
            "底部死区",
            i32::from(b[7] >> 1),
            0,
            50.min(max_normal),
            base,
        ),
    ];
    let switches: Vec<_> = m
        .switches
        .iter()
        .filter(|(_, _, min)| *min <= fw)
        .map(|(v, l, _)| (*v, *l))
        .collect();
    if !switches.is_empty() {
        fields.push(choice(
            "switch",
            "轴体",
            i32::from(b[0] & 15),
            options(&switches),
        ));
    }
    fields
}
fn precision_modes(m: Model, tachyon: bool, fw: u16) -> Vec<u8> {
    let mut out = Vec::new();
    if tachyon {
        for mode in [2u8, 1] {
            if fw >= m.precision_fw[usize::from(mode - 1)] {
                out.push(mode)
            }
        }
    }
    out.push(0);
    out
}
fn set_rt(m: Model, n: i32, b: &mut [u8], idx: usize, shift: u8, tachyon: bool, fw: u16) {
    let mode = precision_modes(m, tachyon, fw)
        .into_iter()
        .find(|mode| *mode == 0 || n * rt_base(m, *mode) <= 500_000)
        .unwrap_or(0);
    let scale = rt_base(m, mode);
    let ticks = ((n * scale + 500) / 1000).clamp(if tachyon { 1 } else { 2 }, 512);
    put_bits(&mut b[3], 3 << shift, mode << shift);
    set9(b, idx, ticks);
}
pub fn edit_rt(m: Model, id: &str, n: i32, b: &mut [u8], tachyon: bool, fw: u16) {
    let base = base(m);
    match id {
        "mode" => put_bits(&mut b[1], 15, n as u8),
        "act" => set9(b, 2, n * base / 1000),
        "switch" => put_bits(&mut b[0], 15, n as u8),
        "press" => set_rt(m, n, b, 4, 3, tachyon, fw),
        "release" => set_rt(m, n, b, 6, 1, tachyon, fw),
        "upper" => put_bits(&mut b[5], 0xfe, ((n * base / 1000) as u8) << 1),
        "lower" => put_bits(&mut b[7], 0xfe, ((n * base / 1000) as u8) << 1),
        _ => {}
    }
}
pub fn page(
    io: &mut impl Io,
    m: Model,
    slot: usize,
    k: usize,
    section: &str,
    changes: &Changes,
    info: &[u8],
) -> Result<Value> {
    let config = Region::new(5, 64 * slot, 64);
    let cfg = config.read(io)?;
    let fw = super::firmware(info);
    let tachyon = cfg[7] & 1 != 0;
    let (region, old, fields) = if section == "performance" {
        let f = fields_performance(m, &cfg, info);
        (config, cfg, f)
    } else {
        let reg = Region::new(160, 1024 * slot + k * 8, 8);
        let data = reg.read(io)?;
        let f = fields_rt(m, &data, tachyon, fw);
        (reg, data, f)
    };
    if changes.is_empty() {
        return Ok(json!({"fields":fields}));
    }
    let mut new = old.clone();
    let mut effective_fields = fields;
    if section == "rt" && changes.contains_key("switch") {
        let one = Changes::from_iter([("switch".into(), changes["switch"].clone())]);
        let selected = validated(&effective_fields, &one)?[0].1;
        edit_rt(m, "switch", selected, &mut new, tachyon, fw);
        effective_fields = fields_rt(m, &new, tachyon, fw);
    }
    for (id, n) in validated(&effective_fields, changes)? {
        if section == "performance" {
            edit_perf(&id, n, &mut new, slot)
        } else {
            edit_rt(m, &id, n, &mut new, tachyon, fw)
        }
    }
    if section == "performance" {
        if let Some(value) = changes.get("never").and_then(Value::as_i64) {
            new[36] = value as u8;
        }
        if new[1] & 3 == 2 && changes.get("win").and_then(Value::as_i64) == Some(1) {
            return Err("Mac 模式下不能锁定 Windows 键".into());
        }
    }
    let edits = if section == "performance" {
        byte_edits(region, &old, &new)
    } else {
        vec![Edit { region, old, new }]
    };
    commit(io, &edits)?;
    page(io, m, slot, k, section, &Changes::new(), info)
}
pub fn byte_edits(region: Region, old: &[u8], new: &[u8]) -> Vec<Edit> {
    old.iter()
        .zip(new)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| Edit {
            region: Region::new(region.cmd, usize::from(region.addr) + i, 1),
            old: vec![*a],
            new: vec![*b],
        })
        .collect()
}
