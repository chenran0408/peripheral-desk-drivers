use super::*;
use peripheral_desk::controls::{Binding, BindingAction};
use std::collections::BTreeMap;

const LABELS: [&str; 6] = ["左键", "右键", "中键", "前进", "后退", "DPI 键"];
const KEYS: u16 = 0x0001;
const SET_KEY: u16 = 0x0101;
pub(super) const MACRO_SLOTS: u8 = 18;
type Map = Vec<Vec<u8>>;

// M HUB supportDpiKeyDevice omits A7 Magnetic, R7 Ultra and K5 V2. A padded sixth
// record is not a physical key, but may still reference shared macro storage.
fn control_count(pid: u16, map: &[Vec<u8>]) -> usize {
    map.len().min(if matches!(pid, 0x403f | 0x4036 | 0x4040) {
        5
    } else {
        6
    })
}

fn record_size(kind: u8) -> usize {
    match kind {
        0x01 | 0x22 => 4,
        0x23 | 0x24 => 8,
        _ => 3,
    }
}

fn parse(data: &[u8]) -> Option<Map> {
    let mut at = 0;
    let mut keys = vec![];
    for _ in LABELS {
        let Some(&kind) = data.get(at) else { break };
        if kind == 0xff {
            break;
        }
        let end = at + record_size(kind);
        keys.push(data.get(at..end)?.to_vec());
        at = end;
    }
    (keys.len() >= 5).then_some(keys)
}

fn read(t: &mut WinTransport, profile: u8) -> Result<Map, String> {
    read_reply(t, KEYS, &[profile, 0, 6], &mut None, |f| {
        parse(payload(f, KEYS)?)
    })
}

pub(super) fn mouse_button(mask: u8) -> Option<u8> {
    [1, 2, 4, 8, 16]
        .iter()
        .position(|&v| v == mask)
        .map(|i| i as u8 + 1)
}
pub(super) fn mouse_mask(button: u8) -> Option<u8> {
    button
        .checked_sub(1)
        .and_then(|n| [1, 2, 4, 8, 16].get(n as usize).copied())
}

fn action(raw: &[u8]) -> BindingAction {
    match raw {
        [0, mask, 0] if mouse_button(*mask).is_some() => {
            BindingAction::Mouse(mouse_button(*mask).unwrap())
        }
        [0xfe, 0, 0] => BindingAction::Disabled,
        [1, 0, 0, 0] => BindingAction::DpiCycle,
        [1, 2, 0, 0] => BindingAction::DpiUp,
        [1, 3, 0, 0] => BindingAction::DpiDown,
        [0x11, key, 0] => BindingAction::Keyboard {
            usage: u16::from(*key),
            modifiers: 0,
        },
        [0x12, mods, 0] => BindingAction::Keyboard {
            usage: 0,
            modifiers: *mods,
        },
        [0x13 | 0x16, mods, key] => BindingAction::Keyboard {
            usage: u16::from(*key),
            modifiers: *mods,
        },
        [0x14, lo, hi] => BindingAction::Consumer(u16::from_le_bytes([*lo, *hi])),
        [0x21, slot, 0] if *slot < MACRO_SLOTS => BindingAction::Macro((*slot).into()),
        other => BindingAction::Unknown(other.to_vec()),
    }
}

fn encode(action: &BindingAction) -> Result<Vec<u8>, String> {
    Ok(match *action {
        BindingAction::Mouse(b) => vec![0, mouse_mask(b).ok_or("按键无效")?, 0],
        BindingAction::Disabled => vec![0xfe, 0, 0],
        BindingAction::DpiCycle => vec![1, 0, 0, 0],
        BindingAction::DpiUp => vec![1, 2, 0, 0],
        BindingAction::DpiDown => vec![1, 3, 0, 0],
        BindingAction::Keyboard {
            usage: 0,
            modifiers: m,
        } if m != 0 => vec![0x12, m, 0],
        BindingAction::Keyboard {
            usage: k,
            modifiers: 0,
        } if (1..=231).contains(&k) => vec![0x11, k as u8, 0],
        BindingAction::Keyboard {
            usage: k,
            modifiers: m,
        } if (1..=231).contains(&k) => vec![0x13, m, k as u8],
        BindingAction::Consumer(usage) => {
            let [lo, hi] = usage.to_le_bytes();
            vec![0x14, lo, hi]
        }
        BindingAction::Macro(n) if n < u16::from(MACRO_SLOTS) => vec![0x21, n as u8, 0],
        _ => return Err("不支持的按键功能".into()),
    })
}

fn assert_left(keys: &[Vec<u8>], idx: usize, replacement: &[u8]) -> Result<(), String> {
    if !keys.iter().enumerate().any(|(i, k)| {
        if i == idx {
            replacement == [0, 1, 0]
        } else {
            k == &[0, 1, 0]
        }
    }) {
        return Err("请至少保留一个左键功能".into());
    }
    Ok(())
}

fn assign(
    t: &mut WinTransport,
    profile: u8,
    idx: usize,
    keys: &[Vec<u8>],
    next: Vec<u8>,
) -> Result<(), String> {
    let original = keys.get(idx).ok_or("该型号没有这个按键")?.clone();
    assert_left(keys, idx, &next)?;
    settings::verified(original, next, |v| {
        let mut args = vec![profile, idx as u8, 0x0b]; // starting index, one record, first and last chunk
        args.extend(v);
        settings::send_check(t, SET_KEY, &args)?;
        let map = read(t, profile)?;
        map.get(idx).cloned().ok_or_else(|| "按键读回缺失".into())
    })
}

fn state(t: &mut WinTransport, keys: &[Vec<u8>], extended: bool, max_events: usize) -> Value {
    let mut macros = BTreeMap::<usize, Value>::new();
    let mut slots = BTreeMap::new();
    let mut issues = Vec::new();
    let bindings: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| Binding {
            control: i as u16,
            layer: 0,
            label: LABELS[i].into(),
            action: action(k),
            editable: true,
        })
        .collect();
    for (i, k) in keys.iter().enumerate() {
        if let BindingAction::Macro(slot) = action(k) {
            let data = slots
                .entry(slot)
                .or_insert_with(|| onboard_macro::definition(t, slot as u8, max_events));
            match data {
                Ok(v) if extended || v["mode"] != "UntilOtherKey" => {
                    macros.insert(i, v.clone());
                }
                Ok(_) => issues.push(format!(
                    "{}使用按其他键停止的宏，请更新主程序以编辑",
                    LABELS[i]
                )),
                Err(e) => issues.push(format!("{}的原宏已保留：{e}", LABELS[i])),
            }
        }
    }
    json!({"bindings":bindings,"binding_types":["mouse","keyboard","consumer","disabled","dpi-cycle","dpi-up","dpi-down"],"colors":[],
        "macros":macros,"macro_supported":max_events>0,"macro_max_events":max_events,"macro_name_max_bytes":54,
        "macro_modes":["Once","WhileHeld","Toggle","UntilOtherKey"],"macro_event_types":["key","delay","mouse","wheel"],
        "notice":issues.join("；")})
}

pub(super) fn handle(t: &mut WinTransport, req: &Value, info: &Info) -> Result<Value, String> {
    let profile = settings::performance(t, &mut None)?.profile();
    if req["op"] == "controls-apply"
        && req
            .get("profile")
            .is_some_and(|p| p.as_u64() != Some(u64::from(profile)))
    {
        return Err("板载配置已切换，请重新读取按键后再应用".into());
    }
    let mut keys = read(t, profile)?;
    let count = control_count(info.pid, &keys);
    let max = onboard_macro::capacity(info);
    if req["op"] == "controls-apply" {
        let payload = &req["action"];
        if let Some(bind) = payload.get("Bind") {
            if bind["layer"].as_u64() != Some(0) {
                return Err("该鼠标在当前板载配置中使用一层按键".into());
            }
            let idx = bind["control"]
                .as_u64()
                .filter(|&i| i < count as u64)
                .ok_or("该型号没有这个按键")? as usize;
            let action: BindingAction =
                serde_json::from_value(bind["action"].clone()).map_err(|e| e.to_string())?;
            let next = encode(&action)?;
            if let BindingAction::Macro(slot) = action {
                onboard_macro::definition(t, slot as u8, max)?;
            }
            assign(t, profile, idx, &keys[..count], next.clone())?;
            keys[idx] = next;
        } else if let Some(m) = payload.get("Macro") {
            let idx = m["control"]
                .as_u64()
                .filter(|&i| i < count as u64)
                .ok_or("该型号没有这个按键")? as usize;
            assert_left(&keys[..count], idx, &[0x21, 0, 0])?;
            let new = onboard_macro::encode(&m["definition"], max)?;
            let mut used = [0u8; MACRO_SLOTS as usize];
            for p in 0..info.profile_count {
                let map = if p == profile {
                    keys.clone()
                } else {
                    read(t, p)?
                };
                for key in map {
                    if let BindingAction::Macro(slot) = action(&key) {
                        used[slot as usize] += 1;
                    }
                }
            }
            if profile >= info.profile_count {
                return Err("无法确认板载配置范围".into());
            }
            let current = match action(&keys[idx]) {
                BindingAction::Macro(n) => Some(n as u8),
                _ => None,
            };
            // Editing a private, readable macro can reuse its own slot. Other profiles'
            // macros and unbound-but-populated slots stay intact.
            let mut candidate = current
                .filter(|&n| used[n as usize] == 1 && onboard_macro::definition(t, n, max).is_ok());
            if candidate.is_none() {
                for n in 0..MACRO_SLOTS {
                    if used[n as usize] == 0 && onboard_macro::empty(t, n)? {
                        candidate = Some(n);
                        break;
                    }
                }
            }
            let slot = candidate.ok_or("没有空白的板载宏位置，现有宏均已保留")?;
            let baseline = onboard_macro::snapshot(t, slot, new.len(), max)?;
            let next = vec![0x21, slot, 0];
            onboard_macro::replace(t, slot, &baseline, &new)?;
            if let Err(e) = assign(t, profile, idx, &keys[..count], next.clone()) {
                return match onboard_macro::write_and_confirm(t, slot, &baseline) {
                    Ok(()) => Err(format!("{e}；宏存储内容已恢复并读回")),
                    Err(r) => Err(format!("{e}；宏存储内容恢复未确认：{r}")),
                };
            }
            keys[idx] = next;
        } else {
            return Err("鼠标没有这个控制操作".into());
        }
    }
    let mut result = state(
        t,
        &keys[..count],
        req["extended_modes"].as_bool() == Some(true),
        max,
    );
    result["profile"] = json!(profile);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dpi_steps_use_the_vendor_record_and_do_not_expose_a_hidden_sixth_key() {
        for (bytes, binding) in [
            ([1, 0, 0, 0], BindingAction::DpiCycle),
            ([1, 2, 0, 0], BindingAction::DpiUp),
            ([1, 3, 0, 0], BindingAction::DpiDown),
        ] {
            assert_eq!(action(&bytes), binding);
            assert_eq!(encode(&binding).unwrap(), bytes);
        }
        let keys = vec![
            vec![0, 2, 0],
            vec![0, 2, 0],
            vec![0, 4, 0],
            vec![0, 8, 0],
            vec![0, 16, 0],
            vec![0, 1, 0],
        ];
        for pid in [0x403f, 0x4036, 0x4040] {
            let count = control_count(pid, &keys);
            assert_eq!(count, 5);
            assert!(assert_left(&keys[..count], 0, &[1, 2, 0, 0]).is_err());
        }
        assert_eq!(control_count(0x4038, &keys), 6);
        assert_eq!(control_count(0x4038, &keys[..5]), 5);
    }

    #[test]
    fn variable_width_keys_keep_unrecognized_bindings_and_absent_dpi_key() {
        let raw = [
            0, 1, 0, 0x23, 9, 8, 7, 6, 5, 4, 3, 0x13, 1, 6, 0x14, 0xe9, 0, 1, 0, 0, 0, 0xff,
        ];
        let keys = parse(&raw).unwrap();
        assert_eq!(keys.len(), 5);
        assert_eq!(
            action(&keys[1]),
            BindingAction::Unknown(raw[3..11].to_vec())
        );
        assert_eq!(encode(&action(&keys[2])).unwrap(), [0x13, 1, 6]);
        assert_eq!(action(&keys[3]), BindingAction::Consumer(0xe9));
        assert!(assert_left(&keys, 0, &[0xfe, 0, 0]).is_err());
    }
}
