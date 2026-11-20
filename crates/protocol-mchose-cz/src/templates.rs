//! Local magnetic profiles. Exported files keep the model identity. Reads and
//! writes address the selected board's active slot, not the slot in the file.
use super::*;
use wire::{commit, Edit, Region};

struct Block {
    name: &'static str,
    region: Region,
    old: Vec<u8>,
    data: Vec<u8>,
}
fn block(io: &mut impl Io, name: &'static str, region: Region) -> Result<Block> {
    let old = region.read(io)?;
    Ok(Block {
        name,
        region,
        data: old.clone(),
        old,
    })
}
fn firmware_id(v: &Value) -> Option<u16> {
    v.as_u64().and_then(|a| u16::try_from(a).ok()).or_else(|| {
        v.as_str().and_then(|s| {
            s.strip_prefix("0x")
                .map_or_else(|| s.parse().ok(), |h| u16::from_str_radix(h, 16).ok())
        })
    })
}
fn byte(v: &Value) -> Result<u8> {
    v.as_u64()
        .and_then(|a| u8::try_from(a).ok())
        .ok_or_else(|| "模板字节值超出范围".into())
}
fn keyrec(v: &Value) -> Result<[u8; 3]> {
    Ok([byte(&v["type"])?, byte(&v["code1"])?, byte(&v["code2"])?])
}
fn source_array(src: &Value, length: usize) -> Result<Option<Vec<u8>>> {
    if src.is_null() {
        return Ok(None);
    }
    let a = src.as_array().ok_or("模板区段需要字节数组")?;
    if a.len() != length {
        return Err("模板区段与该型号的板载大小不符".into());
    }
    a.iter().map(byte).collect::<Result<_>>().map(Some)
}
fn known(tag: u8) -> bool {
    matches!(tag, 0 | 16 | 32 | 48 | 64 | 144..=149 | 240)
}
pub(super) fn replaceable(old: &[u8]) -> bool {
    known(old[0]) || old == [255; 3]
}
fn slot_ref(rec: &[u8]) -> Option<(usize, usize)> {
    if rec[1] >= 32 {
        return None;
    }
    let bank = match rec[0] {
        144 => 0,
        145 => 2,
        146..=149 => 1,
        _ => return None,
    };
    Some((bank, usize::from(rec[1])))
}
fn apply_action(old: &[u8], dst: &mut [u8], bytes: &[u8]) -> bool {
    if known(bytes[0]) && replaceable(old) {
        dst.copy_from_slice(bytes);
        true
    } else {
        false
    }
}
fn vendor_block(
    b: &Block,
    doc: &Value,
    layer: usize,
    selected: &[usize],
) -> Result<Option<Vec<u8>>> {
    let mut raw = b.old.clone();
    match b.name {
        "travel" => {
            let Some(keys) = doc["triggerTravel"]["travelKeys"].as_array() else {
                return Ok(None);
            };
            for &i in selected {
                let Some(key) = keys.get(i) else { continue };
                let dest = &mut raw[8 * i..8 * i + 8];
                for (prop, idx) in [("keyActuation", 2), ("rtPress", 4), ("rtRelease", 6)] {
                    if let Some(n) = key[prop].as_u64() {
                        if !(1..=512).contains(&n) {
                            return Err(format!("触发值 {prop} 超出范围"));
                        }
                        let [lo, hi] = ((n - 1) as u16).to_le_bytes();
                        dest[idx] = lo;
                        put_bits(&mut dest[idx + 1], 1, hi);
                    }
                }
                for (prop, idx, shift, max) in [
                    ("switchType", 0, 0, 15),
                    ("keyMode", 1, 0, 2),
                    ("priority", 1, 4, 3),
                    ("pressPrecision", 3, 3, 2),
                    ("releasePrecision", 3, 1, 2),
                    ("pressDeadzone", 5, 1, 50),
                    ("releaseDeadzone", 7, 1, 50),
                ] {
                    if let Some(n) = key[prop].as_u64() {
                        if n > max {
                            return Err(format!("触发字段 {prop} 超出范围"));
                        }
                        let mask = match prop {
                            "switchType" | "keyMode" | "priority" => 15,
                            "pressPrecision" | "releasePrecision" => 3,
                            _ => 127,
                        };
                        put_bits(&mut dest[idx], mask << shift, (n as u8) << shift);
                    }
                }
            }
        }
        "matrix" => {
            let Some(keys) = doc["userKeys"].get(layer).and_then(Value::as_array) else {
                return Ok(None);
            };
            for &i in selected {
                if let Some(k) = keys.get(i) {
                    raw[3 * i..3 * i + 3].copy_from_slice(&keyrec(k)?);
                }
            }
        }
        "performance" => {
            let cfg = &doc["performance"];
            if !cfg.is_object() {
                return Ok(None);
            }
            for (prop, idx, mask, shift) in [
                ("reporteRate", 4, 15, 0),
                ("macMode", 1, 15, 0),
                ("tachyonMode", 7, 1, 0),
                ("lockWin", 6, 1, 0),
                ("bottomrapidtriggerMode", 7, 2, 1),
                ("travelCheck", 7, 4, 2),
                ("debounceLevel", 7, 224, 5),
                ("rtsmart", 20, 255, 0),
                ("sleepTime", 35, 255, 0),
                ("sleepMode", 36, 255, 0),
            ] {
                if let Some(v) = cfg.get(prop) {
                    let n = byte(v)?;
                    if n > (mask >> shift) {
                        return Err(format!("性能字段 {prop} 超出范围"));
                    }
                    put_bits(&mut raw[idx], mask, n << shift);
                }
            }
        }
        _ => {
            let Some(records) = doc["advancedKeys"][b.name].as_array() else {
                return Ok(None);
            };
            if records.len() > 32 {
                return Err("高级键槽数量超过 32".into());
            }
            let stride = match b.name {
                "dks" => 24,
                "mt" => 6,
                _ => 3,
            };
            for (i, obj) in records.iter().enumerate() {
                let data = &mut raw[i * stride..(i + 1) * stride];
                match b.name {
                    "dks" => {
                        let pts = obj["point"].as_array().ok_or("DKS 模板缺少四个触发位置")?;
                        if pts.len() != 4 {
                            return Err("DKS 需要四个触发位置".into());
                        }
                        for (j, n) in pts.iter().enumerate() {
                            data[j] = byte(n)?
                        }
                        for j in 0..4 {
                            let key = keyrec(&obj[format!("action{j}")])?;
                            let offset = 4 + 5 * j;
                            data[offset..offset + 3].copy_from_slice(&key);
                            let mut flags = 0u16;
                            for k in 0..4 {
                                let n = byte(&obj[format!("point{j}")][format!("action{k}")])?;
                                if n > if k == 3 { 1 } else { 7 } {
                                    return Err("DKS 动作阶段无效".into());
                                }
                                flags |= u16::from(n) << (3 * k);
                            }
                            data[offset + 3..offset + 5].copy_from_slice(&flags.to_le_bytes());
                        }
                    }
                    "mt" => {
                        data[..3].copy_from_slice(&keyrec(&obj["clickKey"])?);
                        data[3..].copy_from_slice(&keyrec(&obj["downKey"])?);
                    }
                    _ => data.copy_from_slice(&keyrec(obj)?),
                }
            }
        }
    }
    Ok(Some(raw))
}
fn blocks_bank_name(i: usize) -> &'static str {
    ["dks", "mt", "tgl"][i]
}
fn native_export(blocks: &[Block], key: &str, slot: usize) -> Value {
    let mut out = json!({"schema":1,"model":key,"profile":slot,"matrix":[]});
    for b in blocks {
        if b.name == "matrix" {
            out["matrix"]
                .as_array_mut()
                .unwrap()
                .extend(b.old.iter().map(|v| json!(v)));
        } else {
            out[b.name] = json!(b.old);
        }
    }
    out
}
pub fn handle(
    io: &mut impl Io,
    key: &str,
    slot: usize,
    op: &str,
    req: &Value,
    keys: &[Value],
    m: Model,
) -> Result<Value> {
    let is_apply = op.ends_with("apply");
    let src = &req["document"];
    let vendor = src["dataScope"] == "KeyboardProfile";
    let mut limits = Vec::new();
    if is_apply {
        if vendor {
            if src["version"] != 3
                || firmware_id(&src["vendorId"]) != Some(m.vid)
                || firmware_id(&src["productId"]) != Some(m.pid)
            {
                return Err("模板型号、连接接口或版本与所选键盘不符".into());
            }
            limits.push("原文件中的灯效、宏和个性化设置另外保留");
        } else if src["schema"] != 1 || src["model"] != key {
            return Err("请选择相同型号的磁轴模板".into());
        }
    }
    let mut blocks = vec![
        block(io, "dks", Region::new(162, 768 * slot, 768))?,
        block(io, "mt", Region::new(164, 256 * slot, 192))?,
        block(io, "tgl", Region::new(166, 128 * slot, 96))?,
        block(io, "travel", Region::new(160, 1024 * slot, 1024))?,
        block(io, "performance", Region::new(5, 64 * slot, 64))?,
    ];
    for l in 0..4 {
        blocks.push(block(
            io,
            "matrix",
            Region::new(8, 2048 * slot + 512 * l, 384),
        )?);
    }
    if !is_apply {
        return Ok(native_export(&blocks, key, slot));
    }
    let original = &src["data"];
    let vendor_data = original.get("data").unwrap_or(original);
    let selected: Vec<usize> = keys
        .iter()
        .filter_map(|k| k["value"].as_u64().map(|v| v as usize))
        .collect();
    let native_matrix = if vendor {
        None
    } else {
        source_array(&src["matrix"], 1536)?
    };
    let mut skipped = false;
    let mut refs = [[false; 32]; 3];
    let mut imported = Vec::new();
    for (i, b) in blocks.iter().enumerate() {
        let l = i.saturating_sub(5);
        let raw = if vendor {
            vendor_block(b, vendor_data, l, &selected)?
        } else if b.name == "matrix" {
            native_matrix
                .as_ref()
                .map(|x| x[384 * l..384 * (l + 1)].to_vec())
        } else {
            source_array(&src[b.name], b.old.len())?
        };
        imported.push(raw);
    }
    // Unchanged keys/layers can share a slot with a template. Never rewrite a
    // slot used by an omitted or unsupported binding.
    let info = Region::new(3, 0, 7).read(io)?;
    let fw = super::firmware(&info);
    let mut protected = [[false; 32]; 3];
    for (idx, b) in blocks.iter_mut().enumerate().skip(5) {
        let layer = idx - 5;
        for k in 0..128 {
            let off = 3 * k;
            let old = &b.old[off..off + 3];
            let rec = imported[idx].as_ref().map(|raw| &raw[off..off + 3]);
            let in_document = selected.contains(&k)
                && (!vendor
                    || vendor_data["userKeys"]
                        .get(layer)
                        .and_then(|a| a.get(k))
                        .is_some());
            let accepted = if let Some(next) = rec.filter(|_| in_document) {
                let mut allowed = known(next[0]) && replaceable(old);
                if (144..=149).contains(&next[0]) {
                    allowed &= slot_ref(next).is_some_and(|(bank, slot)| {
                        if vendor {
                            vendor_data["advancedKeys"][blocks_bank_name(bank)]
                                .get(slot)
                                .is_some()
                        } else {
                            imported[bank].is_some()
                        }
                    });
                    if next[0] >= 147 {
                        let partner = usize::from(next[2]);
                        allowed &= selected.contains(&partner)
                            && partner != k
                            && (!vendor
                                || vendor_data["userKeys"]
                                    .get(layer)
                                    .and_then(|a| a.get(partner))
                                    .is_some())
                            && imported[idx].as_ref().is_some_and(|raw| {
                                let Some(peer) = raw.get(partner * 3..partner * 3 + 3) else {
                                    return false;
                                };
                                peer[0] == next[0]
                                    && usize::from(peer[2]) == k
                                    && replaceable(&b.old[partner * 3..partner * 3 + 3])
                                    && slot_ref(peer).is_some_and(|(bank, slot)| {
                                        !vendor
                                            || vendor_data["advancedKeys"][blocks_bank_name(bank)]
                                                .get(slot)
                                                .is_some()
                                    })
                            });
                    }
                }
                if allowed {
                    if let Some((bank, slot)) = slot_ref(next) {
                        refs[bank][slot] = true;
                    }
                    apply_action(old, &mut b.data[off..off + 3], next)
                } else {
                    skipped = true;
                    false
                }
            } else {
                false
            };
            if !accepted {
                if let Some((bank, slot)) = slot_ref(old) {
                    protected[bank][slot] = true;
                }
            }
        }
    }
    for (idx, b) in blocks.iter_mut().enumerate().take(5) {
        let Some(raw) = &imported[idx] else { continue };
        match b.name {
            "performance" => {
                let fields = settings::fields_performance(m, raw, &info);
                let current = settings::fields_performance(m, &b.old, &info);
                for f in fields {
                    let id = f["id"].as_str().unwrap();
                    let value = f["value"].as_i64().unwrap() as i32;
                    if current.iter().any(|c| c["id"] == id && c["value"] == value) {
                        continue;
                    }
                    let wire_name = match id {
                        "rate" => "reporteRate",
                        "bottom" => "bottomrapidtriggerMode",
                        "stable" => "debounceLevel",
                        "smart" => "rtsmart",
                        "mac" => "macMode",
                        "tachyon" => "tachyonMode",
                        "win" => "lockWin",
                        "never" => "sleepMode",
                        "sleep" => "sleepTime",
                        "check" => "travelCheck",
                        _ => continue,
                    };
                    if vendor && vendor_data["performance"].get(wire_name).is_none() {
                        continue;
                    }
                    if !(f["min"].as_i64().unwrap_or(i64::MIN)
                        ..=f["max"].as_i64().unwrap_or(i64::MAX))
                        .contains(&i64::from(value))
                    {
                        skipped = true;
                        continue;
                    }
                    if let Some(c) = f["choices"].as_array() {
                        if !c.iter().any(|k| k["value"] == value) {
                            skipped = true;
                            continue;
                        }
                    }
                    match id {
                        "sleep" => b.data[35] = (value / 30) as u8,
                        "mac" => put_bits(&mut b.data[1], 15, 4 * slot as u8 + 2 * value as u8),
                        _ => settings::edit_perf(id, value, &mut b.data, slot),
                    }
                }
            }
            "travel" => {
                for &k in &selected {
                    let n = 8 * k;
                    let switch = i32::from(raw[n] & 15);
                    let switch_ok = m
                        .switches
                        .iter()
                        .any(|(v, _, min)| *v == switch && *min <= fw);
                    let mut masks = [
                        if switch_ok { 15u8 } else { 0 },
                        0xff,
                        0xff,
                        0x1f,
                        0xff,
                        0xff,
                        0xff,
                        0xff,
                    ];
                    if raw[n + 1] & 15 > 2 {
                        masks[1] &= 0xf0;
                    }
                    if raw[n + 1] >> 4 > 3 {
                        masks[1] &= 0x0f;
                    }
                    if raw[n + 3] >> 3 & 3 > 2 {
                        masks[3] &= !0x18;
                        masks[4] = 0;
                        masks[5] &= !1;
                    }
                    if raw[n + 3] >> 1 & 3 > 2 {
                        masks[3] &= !0x06;
                        masks[6] = 0;
                        masks[7] &= !1;
                    }
                    if raw[n + 5] >> 1 > 50 {
                        masks[5] &= 1;
                    }
                    if raw[n + 7] >> 1 > 50 {
                        masks[7] &= 1;
                    }
                    skipped |= masks.iter().enumerate().any(|(j, &mask)| {
                        let writable = [15u8, 255, 255, 31, 255, 255, 255, 255][j];
                        (raw[n + j] ^ b.old[n + j]) & (writable & !mask) != 0
                    });
                    for (j, mask) in masks.into_iter().enumerate() {
                        put_bits(&mut b.data[n + j], mask, raw[n + j]);
                    }
                }
            }
            _ => {
                let width = b.data.len() / 32;
                for (n, used) in refs[idx].iter().enumerate() {
                    if !used {
                        continue;
                    }
                    let off = width * n;
                    let mut next = raw[off..off + width].to_vec();
                    let starts: &[usize] = match b.name {
                        "dks" => &[4, 9, 14, 19],
                        "mt" => &[0, 3],
                        _ => &[0],
                    };
                    for &p in starts {
                        if !known(next[p]) {
                            return Err("模板引用的高级键含未知动作，请保留原板载设置".into());
                        }
                        if b.name == "dks" {
                            put_bits(&mut next[p + 4], 0xfc, b.old[off + p + 4]);
                        }
                    }
                    if protected[idx][n] && next != b.old[off..off + width] {
                        return Err(
                            "模板的高级键槽也被其他保留的按键使用，请先解除共享或换一个配置".into(),
                        );
                    }
                    b.data[off..off + width].copy_from_slice(&next);
                }
            }
        }
    }
    if skipped {
        limits.push("不支持的键位、宏引用或回报率已保留");
    }
    let mut edits = Vec::new();
    for b in blocks {
        if b.name == "performance" {
            edits.extend(settings::byte_edits(b.region, &b.old, &b.data));
            continue;
        }
        let stride = match b.name {
            "dks" => 24,
            "mt" => 6,
            "travel" => 8,
            _ => 3,
        };
        let chunk_size = 56 / stride * stride;
        for (i, (a, z)) in b
            .old
            .chunks(chunk_size)
            .zip(b.data.chunks(chunk_size))
            .enumerate()
        {
            if a != z {
                edits.push(Edit {
                    region: Region::new(
                        b.region.cmd,
                        usize::from(b.region.addr) + i * chunk_size,
                        a.len(),
                    ),
                    old: a.to_vec(),
                    new: z.to_vec(),
                });
            }
        }
    }
    let changed = commit(io, &edits)?;
    Ok(json!({"applied":true,"changed":changed,"skipped":limits}))
}
