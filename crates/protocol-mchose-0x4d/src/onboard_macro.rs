use super::*;
use peripheral_desk::controls::MacroEvent;

const READ: u16 = 0x090c;
const WRITE: u16 = 0x090b;
const HEADER: usize = 64;
const MAX_UNIT: usize = 4096;

pub(super) fn capacity(info: &Info) -> usize {
    // M HUB caps the newMouse editor at (4096-64)/4. Prefer a smaller unit
    // advertised by the paired mouse, or its shared space when in shared mode.
    let reported = if info.macro_unit != 0 {
        info.macro_unit
    } else {
        info.macro_shared
    };
    let bytes = if reported == 0 {
        MAX_UNIT
    } else {
        usize::from(reported).min(MAX_UNIT)
    };
    bytes.saturating_sub(HEADER) / 4
}

fn read_range(t: &mut WinTransport, slot: u8, start: usize, len: usize) -> Result<Vec<u8>, String> {
    let mut data = Vec::with_capacity(len);
    while data.len() < len {
        let off = start + data.len();
        let [lo, hi] = (off as u16).to_le_bytes();
        let n = (len - data.len()).min(22);
        let args = [slot, lo, hi, n as u8];
        let chunk = read_reply(t, READ, &args, &mut None, |f| {
            let reply = payload(f, READ)?;
            (reply != args)
                .then(|| reply.get(1..1 + n).map(<[u8]>::to_vec))
                .flatten()
        })?;
        data.extend(chunk);
    }
    Ok(data)
}

fn bytes_required(header: &[u8], max: usize) -> Option<usize> {
    let n = usize::from(u16::from_le_bytes([*header.first()?, *header.get(1)?]));
    (n.is_multiple_of(4) && n / 4 <= max).then_some(HEADER + n)
}

pub(super) fn empty(t: &mut WinTransport, slot: u8) -> Result<bool, String> {
    let head = read_range(t, slot, 0, HEADER)?;
    Ok(head.iter().all(|&b| b == 0) || head.iter().all(|&b| b == 0xff))
}

pub(super) fn snapshot(
    t: &mut WinTransport,
    slot: u8,
    new_len: usize,
    max: usize,
) -> Result<Vec<u8>, String> {
    let mut head = read_range(t, slot, 0, 22)?;
    let len = bytes_required(&head, max).unwrap_or(new_len).max(new_len);
    head.extend(read_range(t, slot, head.len(), len - head.len())?);
    Ok(head)
}

fn write(t: &mut WinTransport, slot: u8, data: &[u8]) -> Result<(), String> {
    for (i, chunk) in data.chunks(19).enumerate() {
        let off = i * 19;
        let [lo, hi] = (off as u16).to_le_bytes();
        let flags = u8::from(off == 0) | (u8::from(off + chunk.len() == data.len()) << 1);
        let mut args = vec![slot, lo, hi, ((chunk.len() as u8) << 2) | flags];
        args.extend_from_slice(chunk);
        settings::send_check(t, WRITE, &args)?;
    }
    Ok(())
}

pub(super) fn write_and_confirm(t: &mut WinTransport, slot: u8, data: &[u8]) -> Result<(), String> {
    write(t, slot, data)?;
    if read_range(t, slot, 0, data.len())? != data {
        return Err("板载宏读回与写入不一致".into());
    }
    Ok(())
}

pub(super) fn replace(
    t: &mut WinTransport,
    slot: u8,
    before: &[u8],
    next: &[u8],
) -> Result<(), String> {
    match write_and_confirm(t, slot, next) {
        Ok(()) => Ok(()),
        Err(e) => match write_and_confirm(t, slot, before) {
            Ok(()) => Err(format!("{e}；已恢复并读回宏存储内容")),
            Err(r) => Err(format!("{e}；宏存储内容恢复未确认：{r}")),
        },
    }
}

pub(super) fn device_ref(t: &mut WinTransport, slot: u8) -> Result<String, String> {
    let bytes = read_range(t, slot, 8, HEADER - 8)?;
    let len = bytes.iter().position(|v| *v == 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[..len].to_vec()).map_err(|_| "宏名称编码未知".into())
}
pub(super) fn definition(t: &mut WinTransport, slot: u8, max: usize) -> Result<Value, String> {
    let mut head = read_range(t, slot, 0, 22)?;
    let len = bytes_required(&head, max)
        .filter(|&l| l > HEADER)
        .ok_or("宏长度未知或该位置为空")?;
    head.extend(read_range(t, slot, head.len(), len - head.len())?);
    decode(&head)
}

fn decode(bytes: &[u8]) -> Result<Value, String> {
    let name = bytes.get(8..HEADER).ok_or("宏头不完整")?;
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    let text = std::str::from_utf8(&name[..end]).map_err(|_| "宏名编码未知")?;
    let name = text
        .strip_suffix("$0")
        .or_else(|| text.strip_suffix("$1"))
        .unwrap_or(text);
    let mode = match bytes[4] {
        0 => "WhileHeld",
        1 => "Toggle",
        2 => "Once",
        3 => "UntilOtherKey",
        _ => return Err("设备宏使用未知触发模式".into()),
    };
    let (chunks, remainder) = bytes[HEADER..].as_chunks::<4>();
    if !remainder.is_empty() || chunks.is_empty() {
        return Err("宏事件长度无效".into());
    }
    let mut events = Vec::with_capacity(chunks.len());
    for (i, &[lo, hi, flags, code]) in chunks.iter().enumerate() {
        if ((flags & 0x80) != 0) != (i + 1 == chunks.len()) {
            return Err("宏末尾标志不一致".into());
        }
        let pressed = flags & 0x40 != 0;
        let e = match flags & 0x3f {
            0x3f => MacroEvent::Delay(u16::from_le_bytes([lo, hi])),
            0x11 if code != 0 && code <= 231 => MacroEvent::Key {
                usage: code.into(),
                pressed,
            },
            0 => MacroEvent::Mouse {
                button: buttons::mouse_button(code).ok_or("未知鼠标宏动作")?,
                pressed,
            },
            5 if code < 2 => MacroEvent::Wheel(if code == 0 { 1 } else { -1 }),
            _ => return Err("此宏含其他事件类型，原数据已保留".into()),
        };
        events.push(e);
    }
    Ok(json!({"name":name,"mode":mode,"events":events}))
}

pub(super) fn encode(def: &Value, max: usize) -> Result<Vec<u8>, String> {
    let mode = match def["mode"].as_str() {
        Some("WhileHeld") => 0,
        Some("Toggle") => 1,
        Some("Once") => 2,
        Some("UntilOtherKey") => 3,
        _ => return Err("该鼠标不支持这种宏播放方式".into()),
    };
    let name = def["name"].as_str().ok_or("请填写宏名称")?;
    if name.trim().is_empty() || name.contains('\0') || name.len() > 54 {
        return Err("宏名称长度为 1–54 个 UTF-8 字节".into());
    }
    let events: Vec<MacroEvent> =
        serde_json::from_value(def["events"].clone()).map_err(|e| e.to_string())?;
    if events.is_empty() || events.len() > max {
        return Err(format!("该鼠标支持 1–{max} 个宏事件（含延时）"));
    }
    let mut bytes = vec![0; HEADER];
    bytes[..2].copy_from_slice(&((events.len() * 4) as u16).to_le_bytes());
    bytes[4] = mode;
    bytes[8..8 + name.len()].copy_from_slice(name.as_bytes());
    bytes[8 + name.len()..10 + name.len()].copy_from_slice(b"$0");
    for (i, event) in events.iter().enumerate() {
        let mut rec = match *event {
            MacroEvent::Delay(ms) => {
                let [lo, hi] = ms.to_le_bytes();
                [lo, hi, 0x3f, 0]
            }
            MacroEvent::Key { usage, pressed } if (1..=231).contains(&usage) => {
                [0, 0, 0x11 | ((pressed as u8) << 6), usage as u8]
            }
            MacroEvent::Mouse { button, pressed } => [
                0,
                0,
                (pressed as u8) << 6,
                buttons::mouse_mask(button).ok_or("无效鼠标按钮")?,
            ],
            MacroEvent::Wheel(n) if n == 1 || n == -1 => [0, 0, 5, u8::from(n < 0)],
            _ => return Err("该宏事件超出设备编码范围".into()),
        };
        if i + 1 == events.len() {
            rec[2] |= 0x80;
        }
        bytes.extend_from_slice(&rec);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_hub_macro_modes_and_four_event_types_roundtrip() {
        for mode in ["Once", "WhileHeld", "Toggle", "UntilOtherKey"] {
            let def = json!({"name":"本地","mode":mode,"events":[{"Key":{"usage":4,"pressed":true}},{"Delay":300},{"Mouse":{"button":4,"pressed":false}},{"Wheel":-1}]});
            let bytes = encode(&def, 1008).unwrap();
            assert_eq!(&bytes[..2], &[16, 0]);
            assert_eq!(
                &bytes[HEADER..],
                &[0, 0, 0x51, 4, 44, 1, 0x3f, 0, 0, 0, 0, 8, 0, 0, 0x85, 1]
            );
            assert_eq!(decode(&bytes).unwrap(), def);
        }
    }
}
