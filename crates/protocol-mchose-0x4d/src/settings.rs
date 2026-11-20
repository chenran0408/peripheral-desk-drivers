//! Configuration payloads in M HUB's newMouse protocol.
use super::*;

const PERFORMANCE_WRITE: u16 = 0x0102;
const SINGLE_DPI: u16 = 0x0104;
const LOD: u16 = 0x0009;
const LOD_WRITE: u16 = 0x0109;

#[derive(Clone, Copy)]
struct Model {
    dpi_max: u16,
    lod: &'static [&'static str],
}

impl Model {
    fn lookup(pid: u16) -> Result<Self, String> {
        // The body PID and mouseDiffConfig, matched through the 0x1018 receiver.
        let (dpi_max, lod): (u16, &[&str]) = match pid {
            0x4035 | 0x403b | 0x4030 | 0x4037 => (26000, &["1 mm", "2 mm"]),
            0x4027 | 0x4031 | 0x4032 => (42000, &["0.7 mm", "1 mm", "2 mm"]),
            0x4042 | 0x4034 | 0x403c | 0x4028 | 0x403d | 0x403a | 0x403f | 0x4038 | 0x4036 => {
                (50000, &["0.7 mm", "0.9 mm", "1.2 mm", "1.4 mm", "1.7 mm"])
            }
            _ => return Err("没有该鼠标的性能设置表".into()),
        };
        Ok(Self { dpi_max, lod })
    }

    fn dedicated_lod(self) -> bool {
        self.lod.len() == 5
    }

    fn lod_index(self, sensor: u8) -> Option<u8> {
        ((sensor >> 5) & 3)
            .checked_sub(u8::from(self.lod.len() == 2))
            .filter(|&n| usize::from(n) < self.lod.len())
    }
}

pub(super) fn rate_table(model: u16, receiver: u16) -> Result<Vec<u16>, String> {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../catalog/upstream/vigil-watt/mchose-catalog.json"
    ))
    .expect("checked-in MCHOSE catalog");
    let table = catalog["rateProfiles"][format!("0x{model:04x}")][format!("0x{receiver:04x}")]
        .as_str()
        .ok_or("官方目录没有这组鼠标与接收器的回报率表")?;
    serde_json::from_value(catalog["rateTables"][table].clone()).map_err(|e| e.to_string())
}

#[derive(Clone, Copy)]
pub(super) struct Performance {
    bytes: [u8; 19],
}

impl Performance {
    pub(super) fn from_payload(data: &[u8]) -> Option<Self> {
        if data.len() < 9 || data[0] > 2 {
            return None;
        }
        // M HUB emits a 19-byte block. Preserve every reported reserved byte;
        // only the unreported reserved padding follows the vendor's zero fill.
        let mut bytes = [0; 19];
        let count = bytes.len().min(data.len());
        bytes[..count].copy_from_slice(&data[..count]);
        Some(Self { bytes })
    }

    pub(super) fn profile(self) -> u8 {
        self.bytes[0]
    }

    fn set(&mut self, field: &str, n: i64, model: Model, rates: &[u16]) -> Result<(), String> {
        let sensor = self.bytes[5];
        match field {
            "rate" => {
                let index = rates
                    .iter()
                    .position(|&hz| i64::from(hz) == n)
                    .ok_or("该接收器不支持这个回报率")? as u8;
                let wire = if index == 0 { 0 } else { index + 1 };
                for byte in &mut self.bytes[1..=2] {
                    *byte = (wire << 4) | (*byte & 0x0f);
                }
            }
            "sleep" if (0..=100).contains(&n) => {
                self.bytes[3] = n as u8;
                self.bytes[4] = (self.bytes[4] & !1) | u8::from(n == 0);
            }
            "left_debounce" | "right_debounce" if (1..=20).contains(&n) => {
                self.bytes[if field == "left_debounce" { 7 } else { 8 }] = n as u8;
            }
            "rotation" if (-30..=30).contains(&n) => self.bytes[6] = n as i8 as u8,
            "sensor_mode" if (0..=2).contains(&n) => self.bytes[5] = sensor & !3 | n as u8,
            "ripple" | "line" | "motion_sync" if (0..=1).contains(&n) => {
                let mask = match field {
                    "ripple" => 4,
                    "line" => 8,
                    _ => 16,
                };
                self.bytes[5] = (sensor & !mask) | if n == 1 { mask } else { 0 };
            }
            "lod" if !model.dedicated_lod() && (0..model.lod.len() as i64).contains(&n) => {
                self.bytes[5] =
                    (sensor & !0x60) | ((n as u8 + u8::from(model.lod.len() == 2)) << 5);
            }
            _ => return Err("设置名称或数值超出该型号的范围".into()),
        }
        Ok(())
    }
}

pub(super) fn performance(
    t: &mut WinTransport,
    stage: &mut Option<u8>,
) -> Result<Performance, String> {
    read_reply(t, PERFORMANCE, &[], stage, |f| {
        Performance::from_payload(payload(f, PERFORMANCE)?)
    })
}

fn lod_value(t: &mut WinTransport, profile: u8, model: Model) -> Result<u8, String> {
    read_reply(t, LOD, &[profile], &mut None, |f| {
        let n = *payload(f, LOD)?.first()?;
        (usize::from(n) < model.lod.len()).then_some(n)
    })
}

pub(super) fn describe(
    value: &mut Value,
    t: &mut WinTransport,
    pid: u16,
    perf: Performance,
    profile_count: u8,
    layout: Option<(usize, u8, bool)>,
) -> Result<(), String> {
    let model = Model::lookup(pid)?;
    value["pointer_profile"] = json!(perf.profile());
    value["dpi_min"] = json!(200);
    value["dpi_max"] = json!(model.dpi_max);
    value["dpi_step"] = json!(1);
    value["supported_rates"] = json!(rate_table(pid, RECEIVER)?);
    let b = perf.bytes;
    let control = |field: &str, label: &str, v: i64, min: i64, max: i64, unit: &str| {
        json!({
            "field":field,"label":label,"value":v,"min":min,"max":max,"step":1,"unit":unit
        })
    };
    let mut controls = vec![
        control(
            "sleep",
            "休眠时间（0 为永不）",
            if b[4] & 1 != 0 { 0 } else { i64::from(b[3]) },
            0,
            100,
            "分钟",
        ),
        control("left_debounce", "左键消抖", b[7].into(), 1, 20, "ms"),
        control("right_debounce", "右键消抖", b[8].into(), 1, 20, "ms"),
        control(
            "rotation",
            "传感器旋转",
            i64::from(b[6] as i8),
            -30,
            30,
            "°",
        ),
        control(
            "motion_sync",
            "Motion Sync",
            ((b[5] >> 4) & 1).into(),
            0,
            1,
            "",
        ),
        control("line", "直线修正", ((b[5] >> 3) & 1).into(), 0, 1, ""),
        control("ripple", "纹波修正", ((b[5] >> 2) & 1).into(), 0, 1, ""),
        control("sensor_mode", "性能模式", (b[5] & 3).into(), 0, 2, ""),
    ];
    if let Some(mode) = controls.last_mut() {
        mode["choices"] = choices(&["性能模式", "电竞模式", "超竞模式"]);
    }
    if profile_count > 1 && perf.profile() < profile_count {
        let mut c = control(
            "board_profile",
            "板载配置",
            perf.profile().into(),
            0,
            i64::from(profile_count - 1),
            "",
        );
        c["choices"] = numbered("配置", profile_count);
        controls.push(c);
    }
    if let Some((count, idx, split)) = layout {
        let mut c = control(
            "stage_index",
            "当前 DPI 档位",
            idx.into(),
            0,
            count as i64 - 1,
            "",
        );
        c["choices"] = numbered("档位", count as u8);
        controls.push(c);
        controls.push(control(
            "stage_count",
            "启用的 DPI 档数",
            count as i64,
            1,
            6,
            "",
        ));
        controls.push(control(
            "separate_axes",
            "X/Y 轴独立设置",
            i64::from(split),
            0,
            1,
            "",
        ));
    }
    if model.dpi_max != 26000 {
        controls.push(control(
            "glass",
            "玻璃表面模式",
            i64::from((b[5] >> 7) & 1),
            0,
            1,
            "",
        ));
    }
    let lod = if model.dedicated_lod() {
        lod_value(t, perf.profile(), model).map(Some)
    } else {
        Ok(model.lod_index(b[5]))
    };
    if let Ok(Some(n)) = lod {
        let mut entry = control(
            "lod",
            "抬升距离",
            n.into(),
            0,
            model.lod.len() as i64 - 1,
            "",
        );
        entry["choices"] = choices(model.lod);
        controls.push(entry);
    }
    for entry in &mut controls {
        if matches!(
            entry["field"].as_str(),
            Some("motion_sync" | "line" | "ripple" | "glass" | "separate_axes")
        ) {
            entry["choices"] = choices(&["关闭", "开启"]);
        }
    }
    value["pointer_controls"] = json!(controls);
    lod.map(|_| ())
}

fn numbered(label: &str, count: u8) -> Value {
    Value::Array(
        (0..count)
            .map(|v| json!({"value":v,"label":format!("{label} {}",v+1)}))
            .collect(),
    )
}

fn choices(labels: &[&str]) -> Value {
    Value::Array(
        labels
            .iter()
            .enumerate()
            .map(|(v, label)| json!({"value":v,"label":label}))
            .collect(),
    )
}

pub(super) fn send_check(t: &mut WinTransport, command: u16, data: &[u8]) -> Result<(), String> {
    read_reply(t, command, data, &mut None, |f| {
        payload(f, command).map(|_| ())
    })
}

#[derive(Clone, PartialEq, Eq)]
struct DpiLayout {
    x: DpiConfig,
    y: Option<DpiConfig>,
}

fn axis_payload(profile: u8, axis: u8, cfg: &DpiConfig) -> [u8; 17] {
    let mut data = [0; 17];
    data[..5].copy_from_slice(&[
        profile,
        axis,
        u8::from(cfg.separate_axes),
        cfg.stages.len() as u8,
        cfg.selected,
    ]);
    for (dst, value) in data[5..].as_chunks_mut::<2>().0.iter_mut().zip(cfg.slots) {
        dst.copy_from_slice(&value.to_le_bytes());
    }
    data
}

impl DpiLayout {
    fn read(t: &mut WinTransport, profile: u8) -> Result<Self, String> {
        let x = read_axis(t, profile, 0)?;
        let y = if x.separate_axes {
            Some(read_axis(t, profile, 1)?)
        } else {
            None
        };
        Ok(Self { x, y })
    }

    fn edit(&self, field: &str, n: i64, model: Model) -> Result<Self, String> {
        let mut next = self.clone();
        if field == "separate_axes" {
            match n {
                0 => {
                    next.x.separate_axes = false;
                    next.y = None
                }
                1 if !next.x.separate_axes => {
                    // The official editor initializes Y from X when split mode is first enabled.
                    next.x.separate_axes = true;
                    next.y = Some(next.x.clone());
                }
                1 => {}
                _ => return Err("请选择开启或关闭".into()),
            }
            return Ok(next);
        }
        for axis in std::iter::once(&mut next.x).chain(next.y.iter_mut()) {
            match field {
                "stage_index" if (0..axis.stages.len() as i64).contains(&n) => {
                    axis.selected = n as u8
                }
                "stage_count" if (1..=6).contains(&n) => {
                    let count = n as usize;
                    let defaults = [400, 800, 1600, 3200, 6400, model.dpi_max];
                    for (slot, default) in axis
                        .slots
                        .iter_mut()
                        .zip(defaults)
                        .take(count)
                        .skip(axis.stages.len())
                    {
                        if *slot == 0 {
                            *slot = default;
                        }
                    }
                    axis.stages = axis.slots[..count].to_vec();
                    axis.selected = axis.selected.min(n as u8 - 1);
                }
                _ => return Err("所选 DPI 档位超出范围".into()),
            }
        }
        Ok(next)
    }
}

// Every branch uses the same connected handle for capture, write, readback and rollback.
pub(super) fn verified<T: Eq>(
    before: T,
    desired: T,
    mut send_read: impl FnMut(&T) -> Result<T, String>,
) -> Result<(), String> {
    if before == desired {
        return Ok(());
    }
    let change = |actual: T, expected: &T| {
        if &actual == expected {
            Ok(())
        } else {
            Err("读回值与写入值不一致".to_string())
        }
    };
    match send_read(&desired).and_then(|v| change(v, &desired)) {
        Ok(()) => Ok(()),
        Err(e) => match send_read(&before).and_then(|v| change(v, &before)) {
            Ok(()) => Err(format!("{e}；已恢复并读回原设置")),
            Err(restore) => Err(format!("{e}；恢复原设置未确认：{restore}")),
        },
    }
}

pub(super) fn apply(
    t: &mut WinTransport,
    pid: u16,
    op: &Value,
    profile_count: u8,
) -> Result<Value, String> {
    let field = op["field"].as_str().ok_or("需要设置名称")?;
    let n = op["value"].as_i64().ok_or("需要整数设置值")?;
    let model = Model::lookup(pid)?;
    let before = performance(t, &mut None)?;
    let profile = before.profile();
    if op
        .get("profile")
        .is_some_and(|expected| expected.as_u64() != Some(u64::from(profile)))
    {
        return Err("板载配置已切换，请重新读取后修改".into());
    }
    match field {
        "board_profile" => {
            let new = u8::try_from(n)
                .ok()
                .filter(|idx| *idx < profile_count)
                .ok_or("配置号超出设备报告的数量")?;
            if profile >= profile_count {
                return Err("无法确认当前配置号".into());
            }
            verified(profile, new, |v| {
                send_check(t, 0x0906, &[*v])?;
                std::thread::sleep(Duration::from_millis(100));
                Ok(performance(t, &mut None)?.profile())
            })?;
        }
        "stage_index" | "stage_count" | "separate_axes" => {
            let old = DpiLayout::read(t, profile)?;
            let next = old.edit(field, n, model)?;
            verified(old, next, |layout| {
                for (axis, cfg) in
                    std::iter::once((0, &layout.x)).chain(layout.y.as_ref().map(|y| (1, y)))
                {
                    send_check(t, 0x0103, &axis_payload(profile, axis, cfg))?;
                }
                DpiLayout::read(t, profile)
            })?;
        }
        "dpi" => {
            let next = u16::try_from(n)
                .ok()
                .filter(|v| (200..=model.dpi_max).contains(v))
                .ok_or("DPI 超出该型号的范围")?;
            let axis = match op["axis"].as_str() {
                Some("x") => 0,
                Some("y") => 1,
                _ => return Err("请选择 X 或 Y 轴".into()),
            };
            let index = op["stage"]
                .as_u64()
                .filter(|v| *v < 6)
                .ok_or("DPI 档位超出范围")? as usize;
            if axis == 1 && !read_axis(t, profile, 0)?.separate_axes {
                return Err("当前配置未启用独立 Y 轴".into());
            }
            let slot = |cfg: DpiConfig| {
                cfg.stages
                    .get(index)
                    .copied()
                    .ok_or_else(|| "所选 DPI 档位未启用".to_string())
            };
            let old = slot(read_axis(t, profile, axis)?)?;
            verified(old, next, |v| {
                let [lo, hi] = v.to_le_bytes();
                send_check(t, SINGLE_DPI, &[profile, index as u8, axis, lo, hi])?;
                slot(read_axis(t, profile, axis)?)
            })?;
        }
        "glass" if model.dpi_max != 26000 && (0..=1).contains(&n) => {
            let mut after = before;
            after.bytes[5] = (after.bytes[5] & 0x7f) | (n as u8) << 7;
            let old_lod = if n == 1 && model.dedicated_lod() {
                Some(lod_value(t, profile, model)?)
            } else {
                None
            };
            let highest = (model.lod.len() - 1) as u8;
            if n == 1 && !model.dedicated_lod() {
                after.set("lod", highest.into(), model, &[])?;
            }
            verified(
                (before.bytes, old_lod),
                (after.bytes, old_lod.map(|_| highest)),
                |(bytes, lod)| {
                    send_check(t, PERFORMANCE_WRITE, bytes)?;
                    if let Some(distance) = lod {
                        send_check(t, LOD_WRITE, &[profile, *distance])?;
                    }
                    Ok((
                        performance(t, &mut None)?.bytes,
                        if lod.is_some() {
                            Some(lod_value(t, profile, model)?)
                        } else {
                            None
                        },
                    ))
                },
            )?;
        }
        "lod" if model.dedicated_lod() => {
            let next = u8::try_from(n)
                .ok()
                .filter(|v| usize::from(*v) < model.lod.len())
                .ok_or("抬升档位超出范围")?;
            let old = lod_value(t, profile, model)?;
            verified(old, next, |v| {
                send_check(t, LOD_WRITE, &[profile, *v])?;
                lod_value(t, profile, model)
            })?;
        }
        _ => {
            let mut after = before;
            after.set(field, n, model, &rate_table(pid, RECEIVER)?)?;
            verified(before.bytes, after.bytes, |bytes| {
                send_check(t, PERFORMANCE_WRITE, bytes)?;
                Ok(performance(t, &mut None)?.bytes)
            })?;
        }
    }
    Ok(json!({"field":field,"value":n,"profile":profile}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_write_only_replaces_the_upper_nibbles_and_preserves_unknowns() {
        let input = [
            2, 0x31, 0x14, 8, 0xb0, 0xa9, 3, 5, 7, 12, 18, 5, 7, 11, 23, 0, 0, 1, 6,
        ];
        let mut perf = Performance::from_payload(&input).unwrap();
        let spec = Model::lookup(0x403d).unwrap();
        perf.set("rate", 8000, spec, &rate_table(0x403d, RECEIVER).unwrap())
            .unwrap();
        let mut expected = input;
        expected[1] = 0x61;
        expected[2] = 0x64;
        assert_eq!(perf.bytes, expected);
        assert!(perf.set("rate", 250, spec, &[125, 500, 1000]).is_err());
        perf.set("line", 0, spec, &[]).unwrap();
        assert_eq!(perf.bytes[5], 0xa1);
        perf.set("sleep", 0, spec, &[]).unwrap();
        assert_eq!(&perf.bytes[3..5], &[0, 0xb1]);
    }

    #[test]
    fn vendor_dpi_read_and_write_layouts_reorder_three_header_fields() {
        let frame = query(
            DPI,
            &[2, 0, 2, 1, 0, 0x90, 1, 0x20, 3, 0, 0, 0, 0, 0, 0, 0, 0],
        );
        let snapshot = DpiLayout {
            x: dpi_config(&frame, 2, 0).unwrap(),
            y: None,
        };
        let spec = Model::lookup(0x4035).unwrap();
        let next = snapshot
            .edit("stage_count", 3, spec)
            .unwrap()
            .edit("separate_axes", 1, spec)
            .unwrap();
        assert_eq!(
            &axis_payload(2, 1, next.y.as_ref().unwrap())[..11],
            &[2, 1, 1, 3, 1, 0x90, 1, 0x20, 3, 0x40, 6]
        );
        assert_eq!(snapshot.x.slots[2], 0);
        let next = next.edit("stage_count", 1, spec).unwrap();
        assert_eq!((next.x.selected, next.y.unwrap().selected), (0, 0));
    }

    #[test]
    fn partial_write_is_restored_and_failed_restore_is_reported() {
        let mut sends = Vec::new();
        let e = verified(8, 12, |n| {
            sends.push(*n);
            Ok(if *n == 12 { 9 } else { *n })
        })
        .unwrap_err();
        assert_eq!(sends, [12, 8]);
        assert!(e.contains("已恢复并读回"));
        let failed = verified(8, 12, |_| Err("断开".into())).unwrap_err();
        assert!(failed.contains("恢复原设置未确认"));
    }
}
