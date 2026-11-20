use super::*;
use wire::{commit, Edit, Region};

#[derive(Clone, Copy)]
struct Kind {
    code: u8,
    mem: u8,
    stride: usize,
    bank: usize,
}
impl Kind {
    fn get(section: &str) -> Result<Self> {
        let (code, mem, stride, bank) = match section {
            "dks" => (144, 162, 24, 768),
            "tgl" => (145, 166, 3, 128),
            "mt" => (146, 164, 6, 256),
            "rs" => (147, 164, 6, 256),
            "socd" => (148, 164, 6, 256),
            "lt" => (149, 164, 6, 256),
            _ => return Err("未知高级键".into()),
        };
        Ok(Self {
            code,
            mem,
            stride,
            bank,
        })
    }
    fn paired(self) -> bool {
        self.code >= 147
    }
    fn shares(self, code: u8) -> bool {
        if self.mem == 164 {
            (146..=149).contains(&code)
        } else {
            self.code == code
        }
    }
    fn region(self, profile: usize, slot: usize) -> Region {
        Region::new(
            self.mem,
            self.bank * profile + self.stride * slot,
            self.stride,
        )
    }
}
pub fn matrix(profile: usize, layer: usize, index: usize) -> Region {
    Region::new(8, 2048 * profile + 512 * layer + 3 * index, 3)
}
fn action(buf: &[u8]) -> i32 {
    (i32::from(buf[0]) << 16) | (i32::from(buf[1]) << 8) | i32::from(buf[2])
}
fn encode(n: i32) -> [u8; 3] {
    [(n >> 16) as u8, (n >> 8) as u8, n as u8]
}
fn key_field(id: &str, label: &str, raw: &[u8], keys: &[Value]) -> Value {
    let val = action(raw);
    let mut list = vec![json!({"value":0,"label":"无"})];
    for k in keys {
        if let Some(a) = k["action"].as_i64() {
            if a != 0 && !list.iter().any(|x| x["value"].as_i64() == Some(a)) {
                list.push(json!({"value":a,"label":k["label"]}));
            }
        }
    }
    if val != 0 && !list.iter().any(|x| x["value"] == val) {
        list.push(json!({"value":val,"label":format!("当前动作 {val:06X}")}));
    }
    choice(id, label, val, Value::Array(list))
}
fn allowed_dks(point: usize) -> Value {
    let labels = ["无", "点按", "保持至下一段", "保持至后续段"];
    let values = if point == 3 {
        &[0, 1][..]
    } else if point == 2 {
        &[0, 1, 3][..]
    } else {
        &[0, 1, 3, 7][..]
    };
    Value::Array(
        values
            .iter()
            .zip(labels)
            .map(|(v, l)| json!({"value":v,"label":l}))
            .collect(),
    )
}
fn form(
    kind: Kind,
    enabled: bool,
    bind: &[u8],
    record: &[u8],
    rt: &[u8],
    keys: &[Value],
    index: usize,
) -> Vec<Value> {
    let mut f = vec![bit("enabled", "绑定到所选键", enabled)];
    match kind.code {
        144 => {
            for i in 0..4 {
                let mut field = number(
                    &format!("depth{i}"),
                    ["第一段按下", "第二段按下", "第一段抬起", "第二段抬起"][i],
                    record[i].into(),
                    1,
                    40,
                );
                field["scale"] = json!(10);
                field["unit"] = json!("mm");
                f.push(field);
            }
            for i in 0..4 {
                let start = 4 + 5 * i;
                f.push(key_field(
                    &format!("key{i}"),
                    &format!("动作 {}", i + 1),
                    &record[start..start + 3],
                    keys,
                ));
                let flags = u16::from_le_bytes([record[start + 3], record[start + 4]]);
                for p in 0..4 {
                    let v = i32::from((flags >> (p * 3)) & if p == 3 { 1 } else { 7 });
                    let mut opts = allowed_dks(p);
                    if !opts.as_array().unwrap().iter().any(|c| c["value"] == v) {
                        opts.as_array_mut()
                            .unwrap()
                            .push(json!({"value":v,"label":"保留当前阶段"}));
                    }
                    f.push(choice(
                        &format!("phase{i}_{p}"),
                        &format!(
                            "动作 {} · {}",
                            i + 1,
                            ["按下①", "按下②", "抬起①", "抬起②"][p]
                        ),
                        v,
                        opts,
                    ));
                }
            }
        }
        145 => f.push(key_field("key0", "切换动作", record, keys)),
        146 => {
            f.push(key_field("key0", "轻点动作", record, keys));
            f.push(key_field("key1", "长按动作", &record[3..], keys));
            let mut timer = number(
                "hold",
                "长按阈值",
                if enabled {
                    i32::from(bind[2]) * 10
                } else {
                    150
                },
                10,
                2550,
            );
            timer["unit"] = json!("ms");
            timer["step"] = json!(10);
            f.push(timer);
        }
        _ => {
            let mut partners = vec![json!({"value":-1,"label":"选择第二个按键"})];
            partners.extend(
                keys.iter()
                    .filter(|k| k["value"] != index)
                    .map(|k| json!({"value":k["value"],"label":k["label"]})),
            );
            f.push(choice(
                "partner",
                "第二个按键",
                if enabled { bind[2].into() } else { -1 },
                Value::Array(partners),
            ));
            if kind.code == 148 {
                f.push(choice(
                    "priority",
                    "冲突处理",
                    (rt[1] >> 4).into(),
                    options(&[
                        (0, "最后输入优先"),
                        (1, "按键 1 优先"),
                        (2, "按键 2 优先"),
                        (3, "抵消"),
                    ]),
                ));
            }
        }
    }
    f
}
fn add(io: &mut impl Io, region: Region, new: Vec<u8>, edits: &mut Vec<Edit>) -> Result<()> {
    let old = region.read(io)?;
    edits.push(Edit { region, old, new });
    Ok(())
}
fn set_priority(
    io: &mut impl Io,
    prof: usize,
    k: usize,
    n: u8,
    edits: &mut Vec<Edit>,
) -> Result<()> {
    let region = Region::new(160, prof * 1024 + k * 8, 8);
    let old = region.read(io)?;
    let mut new = old.clone();
    put_bits(&mut new[1], 0xf0, n << 4);
    edits.push(Edit { region, old, new });
    Ok(())
}
fn reset_key(
    io: &mut impl Io,
    prof: usize,
    layer: usize,
    k: usize,
    edits: &mut Vec<Edit>,
) -> Result<()> {
    let r = matrix(prof, layer, k);
    let original = Region { cmd: 7, ..r }.read(io)?;
    add(io, r, original, edits)
}
struct Slots {
    used: [usize; 32],
    contents: Vec<u8>,
}
impl Slots {
    fn read(io: &mut impl Io, prof: usize, kind: Kind) -> Result<Self> {
        let mut used = [0usize; 32];
        for layer in 0..4 {
            let r = Region::new(8, 2048 * prof + 512 * layer, 384);
            for k in r.read(io)?.as_chunks::<3>().0 {
                if kind.shares(k[0]) && k[1] < 32 {
                    used[usize::from(k[1])] += 1;
                }
            }
        }
        let contents = Region::new(kind.mem, prof * kind.bank, 32 * kind.stride).read(io)?;
        Ok(Self { used, contents })
    }
    fn allocate(&mut self, kind: Kind, old: Option<u8>) -> Result<usize> {
        if let Some(i) = old.filter(|i| *i < 32 && self.used[usize::from(*i)] == 1) {
            return Ok(usize::from(i));
        }
        let i = (0..32)
            .find(|&i| {
                self.used[i] == 0
                    && self.contents[i * kind.stride..(i + 1) * kind.stride]
                        .iter()
                        .all(|b| *b == 0)
            })
            .ok_or("没有空闲高级键槽。已有设置已保留。")?;
        self.used[i] = 1;
        Ok(i)
    }
}
fn layer_selector(layer: usize) -> Value {
    json!({"label":"键位层","value":layer,"choices":options(&[(0,"Windows"),(1,"Windows Fn"),(2,"Mac"),(3,"Mac Fn")])})
}
fn combo(
    io: &mut impl Io,
    profile: usize,
    layer: usize,
    index: usize,
    changes: &Changes,
    keys: &[Value],
) -> Result<Value> {
    let region = matrix(profile, layer, index);
    let old = region.read(io)?;
    let bound = old[0] == 16 && old[1] != 0 && old[2] != 0;
    let mut modifiers = options(&[
        (1, "左 Ctrl"),
        (2, "左 Shift"),
        (4, "左 Alt"),
        (8, "左 Win"),
        (16, "右 Ctrl"),
        (32, "右 Shift"),
        (64, "右 Alt"),
        (128, "右 Win"),
    ]);
    if bound && !old[1].is_power_of_two() {
        modifiers
            .as_array_mut()
            .unwrap()
            .push(json!({"value":old[1],"label":format!("当前修饰键组合 0x{:02X}",old[1])}));
    }
    let mut normal = Vec::new();
    for k in keys {
        let Some(a) = k["action"]
            .as_u64()
            .filter(|a| (0x100001..=0x1000ff).contains(a))
        else {
            continue;
        };
        let code = a & 255;
        if !normal.iter().any(|v: &Value| v["value"] == code) {
            normal.push(json!({"value":code,"label":k["label"]}));
        }
    }
    let original_key = old[0] == 16 && old[2] != 0;
    if original_key && !normal.iter().any(|v| v["value"] == old[2]) {
        normal.push(json!({"value":old[2],"label":format!("当前键 0x{:02X}",old[2])}));
    }
    let mut common = if original_key {
        old[2]
    } else {
        normal
            .first()
            .and_then(|k| k["value"].as_u64())
            .ok_or("此型号没有普通键")? as u8
    };
    let mut hot = if bound { old[1] } else { 1 };
    let fields = vec![
        bit("enabled", "绑定到所选键", bound),
        choice("modifier", "修饰键", hot.into(), modifiers),
        choice("key", "普通键", common.into(), Value::Array(normal)),
    ];
    if changes.is_empty() {
        return Ok(json!({"fields":fields,"scope":layer_selector(layer)}));
    }
    if !templates::replaceable(&old) || (144..=149).contains(&old[0]) {
        return Err("请先在已绑定功能的页面解除此键".into());
    }
    let mut enabled = bound;
    for (id, n) in validated(&fields, changes)? {
        match id.as_str() {
            "enabled" => enabled = n == 1,
            "modifier" => hot = n as u8,
            "key" => common = n as u8,
            _ => unreachable!(),
        }
    }
    let mut edits = Vec::new();
    if enabled {
        edits.push(Edit {
            region,
            old,
            new: vec![16, hot, common],
        });
    } else if bound {
        reset_key(io, profile, layer, index, &mut edits)?;
    } else {
        return Err("请先打开「绑定到所选键」，然后应用".into());
    }
    commit(io, &edits)?;
    combo(io, profile, layer, index, &Changes::new(), keys)
}
pub fn page(
    io: &mut impl Io,
    profile: usize,
    layer: usize,
    index: usize,
    section: &str,
    changes: &Changes,
    keys: &[Value],
) -> Result<Value> {
    if section == "cb" {
        return combo(io, profile, layer, index, changes, keys);
    }
    let kind = Kind::get(section)?;
    let main = matrix(profile, layer, index);
    let bind = main.read(io)?;
    let bound = bind[0] == kind.code;
    if bound && kind.paired() && !keys.iter().any(|key| key["value"] == bind[2]) {
        return Err("键盘当前配对指向此布局之外的按键".into());
    }
    if bound && bind[1] >= 32 {
        return Err("高级键槽超出范围".into());
    }
    let mut record = if bound {
        kind.region(profile, usize::from(bind[1])).read(io)?
    } else {
        vec![0; kind.stride]
    };
    if !bound && kind.code == 144 {
        record[..4].copy_from_slice(&[10, 30, 30, 10]);
    }
    let rt = if kind.code == 148 {
        Region::new(160, profile * 1024 + index * 8, 8).read(io)?
    } else {
        vec![0; 8]
    };
    let fields = form(kind, bound, &bind, &record, &rt, keys, index);
    if changes.is_empty() {
        return Ok(json!({"fields":fields,"scope":layer_selector(layer)}));
    }
    let values = validated(&fields, changes)?;
    if !bound && (!templates::replaceable(&bind) || (144..=149).contains(&bind[0])) {
        return Err("请先在已绑定功能的页面解除此键".into());
    }
    let enable = changes
        .get("enabled")
        .and_then(Value::as_i64)
        .map_or(bound, |v| v == 1);
    if !bound && !enable {
        return Err("请先打开「绑定到所选键」，然后应用".into());
    }
    let mut partner = if bound {
        usize::from(bind[2])
    } else {
        usize::MAX
    };
    let mut hold = if bound { bind[2] } else { 15 };
    let mut priority = rt[1] >> 4;
    for (id, n) in values {
        match id.as_str() {
            "partner" => partner = usize::try_from(n).unwrap_or(usize::MAX),
            "priority" => priority = n as u8,
            "hold" => hold = (n / 10) as u8,
            "enabled" => (),
            _ if id.starts_with("depth") => {
                let i = id[5..].parse::<usize>().unwrap();
                record[i] = n as u8;
            }
            _ if id.starts_with("key") => {
                let i = id[3..].parse::<usize>().unwrap();
                let start = if kind.code == 144 { 4 + i * 5 } else { 3 * i };
                record[start..start + 3].copy_from_slice(&encode(n));
            }
            _ => {
                let (i, p) = id.trim_start_matches("phase").split_once('_').unwrap();
                let i = i.parse::<usize>().unwrap();
                let p = p.parse::<usize>().unwrap();
                let j = 7 + i * 5;
                let mut flags = u16::from_le_bytes([record[j], record[j + 1]]);
                let mask = if p == 3 { 1 } else { 7 };
                flags = (flags & !(mask << (3 * p))) | ((n as u16 & mask) << (3 * p));
                record[j..j + 2].copy_from_slice(&flags.to_le_bytes());
            }
        }
    }
    let mut edits = Vec::new();
    if !enable {
        if !bound {
            return page(io, profile, layer, index, section, &Changes::new(), keys);
        }
        reset_key(io, profile, layer, index, &mut edits)?;
        if kind.paired() {
            let other = usize::from(bind[2]);
            let old = matrix(profile, layer, other).read(io)?;
            if old[0] == kind.code && usize::from(old[2]) == index {
                reset_key(io, profile, layer, other, &mut edits)?;
            }
        }
        // Clear slots only when no other binding references them, just like the vendor editor.
        let slots = Slots::read(io, profile, kind)?;
        for (k, orig) in edits
            .iter()
            .map(|e| (e.old[0], e.old[1]))
            .collect::<Vec<_>>()
        {
            if kind.shares(k) && orig < 32 && slots.used[usize::from(orig)] == 1 {
                add(
                    io,
                    kind.region(profile, usize::from(orig)),
                    vec![0; kind.stride],
                    &mut edits,
                )?;
            }
        }
    } else {
        if !kind.paired()
            && (if kind.code == 144 {
                (0..4).all(|i| action(&record[4 + 5 * i..]) == 0)
            } else {
                record.iter().all(|b| *b == 0)
            })
        {
            return Err("请至少选择一个输出动作".into());
        }
        if kind.code == 144 && (record[0] > record[1] || record[3] > record[2]) {
            return Err("请检查 DKS 两个按下和抬起位置的顺序".into());
        }
        let mut slots = Slots::read(io, profile, kind)?;
        let slot = slots.allocate(kind, bound.then_some(bind[1]))?;
        if kind.paired() {
            if !keys
                .iter()
                .any(|k| k["value"].as_u64() == Some(partner as u64))
                || partner == index
            {
                return Err("请选择两个不同的实体键".into());
            }
            if bound && usize::from(bind[2]) != partner {
                return Err("请先解除原按键对，再选择新的按键对".into());
            }
            let other = matrix(profile, layer, partner);
            let old = other.read(io)?;
            let reciprocal = old[0] == kind.code && usize::from(old[2]) == index;
            if (!templates::replaceable(&old) || (144..=149).contains(&old[0])) && !reciprocal {
                return Err("第二个按键已有高级绑定，请先解除".into());
            }
            let second = slots.allocate(kind, reciprocal.then_some(old[1]))?;
            let a = Region { cmd: 7, ..main }.read(io)?;
            let b = Region { cmd: 7, ..other }.read(io)?;
            add(
                io,
                kind.region(profile, slot),
                [a.clone(), b.clone()].concat(),
                &mut edits,
            )?;
            add(
                io,
                kind.region(profile, second),
                [b, a].concat(),
                &mut edits,
            )?;
            if kind.code == 148 {
                set_priority(io, profile, index, priority, &mut edits)?;
                set_priority(
                    io,
                    profile,
                    partner,
                    match priority {
                        1 => 2,
                        2 => 1,
                        p => p,
                    },
                    &mut edits,
                )?;
            }
            edits.push(Edit {
                region: other,
                old,
                new: vec![kind.code, second as u8, index as u8],
            });
        } else {
            add(io, kind.region(profile, slot), record, &mut edits)?;
        }
        let code2 = if kind.paired() {
            partner as u8
        } else if kind.code == 146 {
            hold
        } else {
            0
        };
        edits.push(Edit {
            region: main,
            old: bind,
            new: vec![kind.code, slot as u8, code2],
        });
    }
    commit(io, &edits)?;
    page(io, profile, layer, index, section, &Changes::new(), keys)
}
