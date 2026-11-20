//! M HUB newMouse board files (readMouseAllInfo, mouseKeyToObj and dpiToObj).
use super::*;
use peripheral_desk::controls::{Binding, BindingAction};
use settings::{Model, Performance};

const KEYS: [&str; 6] = ["left", "right", "mid", "forward", "retreat", "dpi"];

// Exact exported envelope name from the fixed mouseDiffConfig body PID table.
fn model_name(pid: u16) -> Result<&'static str, String> {
    Ok(match pid {
        0x4027 => "MCHOSE K7 V2 Pro+",
        0x4028 => "MCHOSE K7 V2 Ultra+",
        0x4030 => "MCHOSE A7 V3",
        0x4031 => "MCHOSE A7 V3 Pro",
        0x4032 => "MCHOSE A7 V3 Pro+",
        0x4034 => "MCHOSE A5 V3 Ultra+(3955)",
        0x4035 => "MCHOSE A5 V3 Pro",
        0x4036 => "MCHOSE R7 Ultra",
        0x4037 => "MCHOSE K5 Pro",
        0x4038 => "MCHOSE K5 Ultra",
        0x403a => "MCHOSE A7 V3-X",
        0x403b => "MCHOSE K7 V2 Pro",
        0x403c => "MCHOSE K7 V2 Ultra",
        0x403d => "MCHOSE A7 V3 Ultra",
        0x403f => "MCHOSE A7 Magnetic",
        0x4040 => "MCHOSE K5 V2",
        0x4042 => "MCHOSE A5 V3 Ultra",
        _ => return Err("型号缺少原版模板名称".into()),
    })
}

fn key_record(kind: u8, text: &str) -> Option<Vec<u8>> {
    if text.len() != (buttons::record_size(kind) - 1) * 2 || !text.is_ascii() {
        return None;
    }
    let mut tail = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    if !matches!(kind, 0x13 | 0x16) {
        tail.reverse()
    }
    let mut record = vec![kind];
    record.extend(tail);
    Some(record)
}
fn key_text(raw: &[u8]) -> (String, String) {
    let mut tail = raw[1..].to_vec();
    if !matches!(raw[0], 0x13 | 0x16) {
        tail.reverse()
    }
    (
        format!("{:02x}", raw[0]),
        tail.iter().map(|b| format!("{b:02x}")).collect(),
    )
}

fn bindings(record: &Value, count: usize) -> (Value, usize) {
    let mut result = Vec::new();
    let mut skipped = 0;
    for (index, key) in KEYS.iter().take(count).enumerate() {
        let kind = &record[format!("{key}KeyType")];
        if kind.is_null() || kind == "ff" {
            continue;
        }
        let decoded = kind
            .as_str()
            .and_then(|k| u8::from_str_radix(k, 16).ok())
            .and_then(|k| key_record(k, record[format!("{key}KeyVal")].as_str()?));
        let Some(action) = decoded.as_deref().map(buttons::action) else {
            skipped += 1;
            continue;
        };
        if matches!(action, BindingAction::Unknown(_)) {
            skipped += 1;
            continue;
        }
        result.push(Binding {
            control: index as u16,
            layer: 0,
            label: key.to_string(),
            action,
            editable: true,
        });
    }
    let mut macros = json!({});
    for (index, def) in record["peripheralDeskMacros"]
        .as_object()
        .into_iter()
        .flatten()
    {
        if index.parse::<usize>().is_ok_and(|i| i < count)
            && serde_json::from_value::<peripheral_desk::controls::MacroDefinition>(def.clone())
                .is_ok()
        {
            macros[index] = def.clone();
        }
    }
    (json!({"bindings":result,"macros":macros}), skipped)
}

fn rate_index(raw: u8) -> u8 {
    let i = raw >> 4;
    if i > 1 {
        i - 1
    } else {
        i
    }
}
fn number(record: &Value, key: &str, max: u16) -> Result<Option<u16>, String> {
    let Some(v) = record.get(key) else {
        return Ok(None);
    };
    let n = v
        .as_u64()
        .or_else(|| v.as_str()?.parse().ok())
        .filter(|n| *n <= u64::from(max))
        .ok_or_else(|| format!("模板中的 {key} 超出范围"))?;
    Ok(Some(n as u16))
}

#[derive(Clone, PartialEq, Eq)]
struct Pointer {
    performance: [u8; 19],
    x: DpiConfig,
    y: Option<DpiConfig>,
    lod: Option<u8>,
    // Original 000A reply: active mode, switch, rate wire index, sensor mode.
    adaptive: Option<[u8; 3]>,
}
fn adaptive_report(t: &mut WinTransport, profile: u8) -> Result<[u8; 4], String> {
    read_reply(t, 0x000a, &[profile, 0], &mut None, |f| {
        payload(f, 0x000a)?.get(..4)?.try_into().ok()
    })
}
fn adaptive_firmware(t: &mut WinTransport, pid: u16) -> Result<Option<String>, String> {
    let min = match pid {
        0x4037 => 13u32,
        0x4038 => 12,
        _ => return Ok(None),
    };
    read_reply(t, 0x0901, &[0], &mut None, |f| {
        let data = payload(f, 0x0901)?;
        let text = format!("{:02x}{:02x}", data.first()?, data.get(1)?);
        let digits = text
            .trim_start_matches('0')
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        Some(
            digits
                .parse::<u32>()
                .is_ok_and(|v| v >= min)
                .then(|| text.trim_start_matches('0').to_owned()),
        )
    })
}
impl Pointer {
    fn read(
        t: &mut WinTransport,
        pid: u16,
        perf: Performance,
        with_adaptive: bool,
    ) -> Result<Self, String> {
        let profile = perf.profile();
        let x = read_axis(t, profile, 0)?;
        let y = if x.separate_axes {
            Some(read_axis(t, profile, 1)?)
        } else {
            None
        };
        let lod = if Model::lookup(pid)?.dedicated_lod() {
            Some(settings::lod_value(t, profile, Model::lookup(pid)?)?)
        } else {
            None
        };
        Ok(Self {
            performance: perf.bytes,
            x,
            y,
            lod,
            adaptive: if with_adaptive {
                let [_, switch, rate, mode] = adaptive_report(t, profile)?;
                Some([switch, rate, mode])
            } else {
                None
            },
        })
    }
    fn record(&self) -> Value {
        let b = &self.performance;
        let mut v = json!({"profileIndex":b[0],"usbDpiIndex":self.x.selected,"usbRateIndex":rate_index(b[1]),"gRateIndex":rate_index(b[2]),
            "sleep":b[3],"sleepMode":b[4]&1,"sensor":b[5],"rotateVal":b[6],"leftKeyDebounce":b[7],"rightKeyDebounce":b[8],
            "dpiDirection":0,"dpiSum":self.x.stages.len(),"dpiHasY":u8::from(self.x.separate_axes)});
        for (prefix, axis) in
            std::iter::once(("dpi", &self.x)).chain(self.y.as_ref().map(|y| ("dpiY", y)))
        {
            for (i, val) in axis.slots.iter().enumerate() {
                v[format!("{prefix}{i}")] = json!(val)
            }
        }
        if let Some(lod) = self.lod {
            v["lodIndex"] = json!(lod)
        }
        if let Some([sw, rate, mode]) = self.adaptive {
            v["adaptivePowerSwitch"] = json!(sw);
            v["adaptivePowerRateIndex"] = json!(if rate > 1 { rate - 1 } else { rate });
            v["sensorMode"] = json!(mode);
        }
        v
    }
    fn desired(&self, pid: u16, r: &Value) -> Result<Self, String> {
        let model = Model::lookup(pid)?;
        let mut next = self.clone();
        let b = &mut next.performance;
        let max_index = settings::rate_table(pid, receiver_for(pid))?
            .len()
            .saturating_sub(1) as u16;
        for (key, idx) in [("usbRateIndex", 1), ("gRateIndex", 2)] {
            if let Some(n) = number(r, key, max_index)? {
                if rate_index(b[idx]) != n as u8 {
                    b[idx] = (b[idx] & 15) | ((n as u8 + u8::from(n != 0)) << 4)
                }
            }
        }
        for (key, idx, max) in [
            ("sleep", 3, 100),
            ("leftKeyDebounce", 7, 20),
            ("rightKeyDebounce", 8, 20),
        ] {
            if let Some(n) = number(r, key, max)? {
                if idx >= 7 && n == 0 && b[idx] != 0 {
                    return Err("按键消抖须为 1–20 ms".into());
                }
                b[idx] = n as u8
            }
        }
        if let Some(n) = number(r, "sleepMode", 1)? {
            b[4] = (b[4] & !1) | n as u8
        }
        if let Some(n) = number(r, "rotateVal", 255)? {
            if !(-30..=30).contains(&(n as u8 as i8)) && n as u8 != b[6] {
                return Err("旋转角度超出范围".into());
            }
            b[6] = n as u8;
        }
        if let Some(n) = number(r, "sensor", 255)? {
            let mask = if model.glass { 0xff } else { 0x7f };
            let mask = if model.dedicated_lod() {
                mask & !0x60
            } else {
                mask
            };
            let n = n as u8;
            if n & 3 == 3 && b[5] & 3 != 3 {
                return Err("模板含未知传感器模式".into());
            }
            b[5] = (b[5] & !mask) | (n & mask);
        }
        if let Some(lod) = &mut next.lod {
            if let Some(n) = number(r, "lodIndex", model.lod.len() as u16 - 1)? {
                *lod = n as u8
            }
        }
        if let Some([sw, rate, mode]) = &mut next.adaptive {
            if let Some(n) = number(r, "adaptivePowerSwitch", 1)? {
                *sw = n as u8
            }
            if let Some(n) = number(r, "adaptivePowerRateIndex", max_index)? {
                *rate = n as u8 + u8::from(n != 0)
            }
            if let Some(n) = number(r, "sensorMode", 2)? {
                *mode = n as u8
            }
        }
        let count = number(r, "dpiSum", 6)?
            .map(usize::from)
            .unwrap_or(self.x.stages.len());
        let selected = number(r, "usbDpiIndex", 5)?
            .map(|n| n as u8)
            .unwrap_or(self.x.selected);
        if count == 0 || selected as usize >= count {
            return Err("当前 DPI 档超出模板启用的档数".into());
        }
        let split = number(r, "dpiHasY", 1)?.map_or(self.x.separate_axes, |v| v == 1);
        if split && next.y.is_none() {
            next.y = Some(self.x.clone())
        } else if !split {
            next.y = None
        }
        if selected != self.x.selected {
            for byte in &mut b[1..=2] {
                *byte = (*byte & 0xf0) | selected
            }
        }
        for (prefix, axis) in
            std::iter::once(("dpi", &mut next.x)).chain(next.y.as_mut().map(|y| ("dpiY", y)))
        {
            for (i, val) in axis.slots.iter_mut().enumerate() {
                if let Some(n) = number(r, &format!("{prefix}{i}"), u16::MAX)? {
                    if n != *val && !(200..=model.dpi_max).contains(&n) && !(i >= count && n == 0) {
                        return Err(format!("第 {} 档 DPI 超出此型号范围", i + 1));
                    }
                    *val = n;
                }
            }
            if axis.slots[..count].contains(&0) {
                return Err("模板缺少启用档位的 DPI".into());
            }
            axis.stages = axis.slots[..count].to_vec();
            axis.selected = selected;
            axis.separate_axes = split;
        }
        Ok(next)
    }
}

// Writing isolated groups lets the host roll back a completed pointer group together
// with remapped keys. Each group's original bytes are retained around the actual HID write.
fn apply_pointer(
    t: &mut WinTransport,
    pid: u16,
    old: &Pointer,
    next: &Pointer,
) -> Result<(), String> {
    let profile = old.performance[0];
    let write_diff = |t: &mut WinTransport, from: &Pointer, to: &Pointer| -> Result<(), String> {
        if from.performance != to.performance {
            settings::send_check(t, 0x0102, &to.performance)?
        }
        if from.x != to.x || from.y != to.y {
            for (axis, cfg) in std::iter::once((0, &to.x)).chain(to.y.as_ref().map(|y| (1, y))) {
                settings::send_check(t, 0x0103, &settings::axis_payload(profile, axis, cfg))?;
            }
        }
        if let Some(lod) = to.lod.filter(|_| from.lod != to.lod) {
            settings::send_check(t, 0x0109, &[profile, lod])?
        }
        if let Some([s, r, m]) = to.adaptive.filter(|_| from.adaptive != to.adaptive) {
            settings::send_check(t, 0x010a, &[profile, s, r, m])?
        }
        let perf = settings::performance(t, &mut None)?;
        if perf.profile() != profile {
            return Err("板载配置已切换".into());
        }
        let actual = Pointer::read(t, pid, perf, to.adaptive.is_some())?;
        if &actual != to {
            return Err("模板设置读回不一致".into());
        }
        Ok(())
    };
    if old == next {
        return Ok(());
    }
    match write_diff(t, old, next) {
        Ok(()) => Ok(()),
        Err(e) => {
            if settings::performance(t, &mut None).is_ok_and(|p| p.profile() == profile) {
                match write_diff(t, next, old) {
                    Ok(()) => Err(format!("{e}；已恢复原配置并读回")),
                    Err(r) => Err(format!("{e}；恢复原配置未确认：{r}")),
                }
            } else {
                Err(format!("{e}；当前板载槽已变化，原配置恢复未确认"))
            }
        }
    }
}

pub(super) fn handle(t: &mut WinTransport, op: &Value, info: &Info) -> Result<Value, String> {
    let perf = settings::performance(t, &mut None)?;
    let profile = perf.profile();
    if op
        .get("profile")
        .is_some_and(|v| v.as_u64() != Some(profile.into()))
    {
        return Err("当前板载槽已切换，请重新应用模板".into());
    }
    let name = model_name(info.pid)?;
    let capture = op["op"] == "template-read";
    let has_adaptive = capture
        || op["record"].get("adaptivePowerSwitch").is_some()
        || op["record"].get("adaptivePowerRateIndex").is_some()
        || op["record"].get("sensorMode").is_some();
    let firmware = if has_adaptive {
        adaptive_firmware(t, info.pid)?
    } else {
        None
    };
    let adaptive = firmware.is_some();
    let old = Pointer::read(t, info.pid, perf, adaptive)?;
    if op["op"] == "template-settings" {
        let next = old.desired(info.pid, &op["record"])?;
        apply_pointer(t, info.pid, &old, &next)?;
        return Ok(json!({"profile":profile,"verified":true}));
    }
    let keys = buttons::read(t, profile)?;
    let count = buttons::control_count(info.pid, &keys);
    if capture {
        let mut record = old.record();
        record["name"] = op["name"].clone();
        if let Some(version) = firmware {
            record["mouseVersion"] = json!(version);
            record["activeReportMode"] = json!(adaptive_report(t, profile)?[0]);
        }
        let controls = buttons::state(t, &keys[..count], true, onboard_macro::capacity(info));
        let mut refs = json!({});
        for (key, raw) in KEYS.iter().zip(&keys[..count]) {
            let (kind, val) = key_text(raw);
            record[format!("{key}KeyType")] = json!(kind);
            record[format!("{key}KeyVal")] = json!(val);
            if let BindingAction::Macro(slot) = buttons::action(raw) {
                if let Ok(reference) = onboard_macro::device_ref(t, slot as u8) {
                    refs[val] = json!(reference);
                }
            }
        }
        if !controls["macros"].as_object().is_none_or(|v| v.is_empty()) {
            record["peripheralDeskMacros"] = controls["macros"].clone();
        }
        if refs.as_object().is_some_and(|v| !v.is_empty()) {
            record["macroInfo"] = refs;
        }
        return Ok(json!({(name):record}));
    }
    let record = &op["record"];
    let next = old.desired(info.pid, record)?;
    let (mut controls, skipped) = bindings(record, count);
    // Original files refer to a named library macro. Reuse a slot only if its complete
    // reference (including the official source suffix) still matches this physical mouse.
    for (i, key) in KEYS.iter().enumerate().take(count) {
        if record[format!("{key}KeyType")] != "21"
            || controls["macros"].get(i.to_string()).is_some()
        {
            continue;
        }
        let Some(val) = record[format!("{key}KeyVal")].as_str() else {
            continue;
        };
        let Some(reference) = record["macroInfo"]
            .get(val)
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        let Some(BindingAction::Macro(slot)) =
            key_record(0x21, val).as_deref().map(buttons::action)
        else {
            continue;
        };
        if onboard_macro::device_ref(t, slot as u8).as_deref() == Ok(reference) {
            if let Ok(def) = onboard_macro::definition(t, slot as u8, onboard_macro::capacity(info))
            {
                controls["macros"][i.to_string()] = def
            }
        }
    }
    let adaptive_skipped = has_adaptive && !adaptive;
    Ok(
        json!({"controls":controls,"profile":profile,"changed":next!=old,
        "skipped_keys":skipped,"adaptive_skipped":adaptive_skipped}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn baseline() -> Pointer {
        let mut performance = [0x5a; 19];
        performance[..9].copy_from_slice(&[2, 0x32, 0x42, 9, 0x80, 0x14, 0xf0, 6, 8]);
        let slots = [400, 800, 1600, 3200, 6400, 0];
        let x = DpiConfig {
            stages: slots[..3].to_vec(),
            slots,
            selected: 2,
            separate_axes: false,
        };
        Pointer {
            performance,
            x,
            y: None,
            lod: Some(2),
            adaptive: Some([1, 3, 1]),
        }
    }
    #[test]
    fn exported_names_match_official_body_ids() {
        assert_eq!(model_name(0x4034).unwrap(), "MCHOSE A5 V3 Ultra+(3955)");
        assert_eq!(model_name(0x403a).unwrap(), "MCHOSE A7 V3-X");
        assert_eq!(model_name(0x403f).unwrap(), "MCHOSE A7 Magnetic");
        assert!(model_name(0x4026).is_err());
    }

    #[test]
    fn hub_records_preserve_editable_fields_reserved_bytes_and_inactive_slots() {
        let old = baseline();
        let mut r = old.record();
        assert!(old.desired(0x4038, &r).unwrap() == old);
        r["profileIndex"] = json!(0);
        r["dpiHasY"] = json!(1);
        r["dpiSum"] = json!(4);
        r["usbDpiIndex"] = json!(1);
        r["dpiY1"] = json!(1550);
        r["usbRateIndex"] = json!(1);
        r["gRateIndex"] = json!(3);
        r["sleepMode"] = json!(1);
        r["adaptivePowerRateIndex"] = json!(2);
        r["futureFlag"] = json!(34);
        let next = old.desired(0x4038, &r).unwrap();
        assert_eq!(&next.performance[..5], &[2, 0x21, 0x41, 9, 0x81]);
        assert_eq!(&next.performance[9..], &[0x5a; 10]);
        assert_eq!(next.y.as_ref().unwrap().stages[1], 1550);
        assert_eq!(next.x.slots[4], 6400);
        assert_eq!(next.record()["gRateIndex"], 3);
        assert!(old
            .desired(0x4038, &json!({"dpiHasY":1,"dpiY1":65}))
            .is_err());
    }
    #[test]
    fn vendor_keys_use_distinct_endian_rules_and_keep_unknown_or_padded_keys() {
        for raw in [
            vec![0, 8, 0],
            vec![0x13, 2, 4],
            vec![1, 3, 0, 0],
            vec![0x21, 12, 0],
            vec![0x24, 1, 2, 3, 4, 5, 6, 7],
        ] {
            let (kind, text) = key_text(&raw);
            assert_eq!(
                key_record(u8::from_str_radix(&kind, 16).unwrap(), &text).unwrap(),
                raw
            );
        }
        let (keys, skipped) = bindings(
            &json!({"leftKeyType":"13","leftKeyVal":"0204","rightKeyType":"24","rightKeyVal":"01020304050607","dpiKeyType":"00","dpiKeyVal":"0001"}),
            5,
        );
        assert_eq!(keys["bindings"].as_array().unwrap().len(), 1);
        assert_eq!(keys["bindings"][0]["action"]["Keyboard"]["usage"], 4);
        assert_eq!(skipped, 1);
    }
}
