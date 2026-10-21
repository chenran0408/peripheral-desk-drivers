//! AJAZZ AJ179 PRO GearHub-V5 wire codec.
//!
//! Packet facts and the captured v3.03 fixture are adapted from the AGPL-3.0
//! implementation at openmouse/mouse-protocol, commit
//! bac031567b28f2a55c616fc525b03f2d5fa452cc. See docs/AJAZZ-AJ179-PRO.md.

use peripheral_desk::{controls::*, model::State};

pub const DEVICE_KEY: &str = "ajazz-aj179-pro";
pub const VID: u16 = 0x3151;
pub const RECEIVER_PID: u16 = 0x402d;
pub const USB_PID: u16 = 0x4026;
pub const BT_PID: u16 = 0x402c;
pub const FEATURE_BYTES: usize = 64;
/// Windows HIDP_CAPS includes the synthetic report-id byte even though the
/// AJ179's descriptor defines an unnumbered 64-byte payload.
pub const WINDOWS_FEATURE_BYTES: usize = FEATURE_BYTES + 1;
pub const PROFILE: u8 = 0;

pub const GET_FIRMWARE: u8 = 0x80;
pub const GET_DEVICE_ID: u8 = 0x8f;
pub const GET_OPTIONS: u8 = 0xd3;
pub const SET_OPTIONS: u8 = 0x53;
pub const GET_DPI: u8 = 0xd4;
pub const SET_DPI: u8 = 0x54;
pub const GET_KEYS: u8 = 0xd0;
pub const SET_KEY: u8 = 0x50;
pub const DONGLE_SELECT: [u8; 2] = [0xf6, 0x05];
pub const DONGLE_STATUS: u8 = 0xf7;
pub const DONGLE_NOTICE_READ: u8 = 0xfc;

pub const POLLING_RATES: [u16; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];
pub const DPI_MIN: u16 = 50;
pub const DPI_MAX: u16 = 26000;
pub const DPI_STEP: u16 = 50;
pub const DPI_STAGES: usize = 8;
pub const X_OFFSET: usize = 8;
pub const Y_OFFSET: usize = 24;
pub const RGB_OFFSET: usize = 40;
pub const RATE_OFFSET: usize = 9;
pub const DEBOUNCE_OFFSET: usize = 10;
pub const FLAGS_OFFSET: usize = 12;
pub const RIPPLE_FLAG: u8 = 4;
pub const SLEEP_BT_OFFSET: usize = 40;
pub const SLEEP_24G_OFFSET: usize = 44;
pub const LIFT_OFF_OFFSET: usize = 52;
pub const ANGLE_OFFSET: usize = 53;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    pub x: u16,
    pub y: u16,
    pub rgb: [u8; 3],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DpiConfig {
    pub active: u8,
    pub stages: Vec<Stage>,
    pub raw: [u8; FEATURE_BYTES],
}

pub fn encode_command(bytes: &[u8]) -> [u8; FEATURE_BYTES] {
    let mut out = [0; FEATURE_BYTES];
    let n = bytes.len().min(FEATURE_BYTES);
    out[..n].copy_from_slice(&bytes[..n]);
    let sum: u16 = out[..7].iter().map(|v| *v as u16).sum();
    out[7] = 255u8.wrapping_sub(sum as u8);
    out
}
pub fn verify_reply(reply: &[u8], command: u8, echoes: bool) -> Result<(), String> {
    if reply.len() < FEATURE_BYTES {
        return Err(format!("GearHub 回复长度 {}，预期 64", reply.len()));
    }
    if reply.iter().take(FEATURE_BYTES).all(|b| *b == 0) {
        return Err("GearHub 返回全零数据，命令可能未送达鼠标".into());
    }
    if echoes && reply[0] != command {
        return Err(format!(
            "GearHub 回复命令 0x{:02X}，预期 0x{command:02X}",
            reply[0]
        ));
    }
    Ok(())
}
pub fn get_request(command: u8) -> [u8; FEATURE_BYTES] {
    encode_command(&[command, PROFILE])
}
pub fn decode_device_id(reply: &[u8]) -> Result<u32, String> {
    verify_reply(reply, GET_DEVICE_ID, true)?;
    Ok(u32::from_le_bytes(reply[1..5].try_into().unwrap()))
}
pub fn decode_dpi(reply: &[u8]) -> Result<DpiConfig, String> {
    verify_reply(reply, GET_DPI, true)?;
    let count = reply[3] as usize;
    if count == 0 || count > DPI_STAGES {
        return Err(format!("AJ179 PRO 返回未确认的 DPI 档位数量 {count}"));
    }
    if reply[2] as usize >= count {
        return Err(format!("AJ179 PRO 返回未确认的当前档位编号 {}", reply[2]));
    }
    let mut stages = Vec::with_capacity(count);
    for i in 0..count {
        let le = |o| u16::from_le_bytes([reply[o + i * 2], reply[o + i * 2 + 1]]);
        let c = RGB_OFFSET + i * 3;
        stages.push(Stage {
            x: le(X_OFFSET),
            y: le(Y_OFFSET),
            rgb: [reply[c], reply[c + 1], reply[c + 2]],
        });
    }
    Ok(DpiConfig {
        active: reply[2],
        stages,
        raw: reply[..FEATURE_BYTES].try_into().unwrap(),
    })
}
pub fn encode_dpi(config: &DpiConfig, active: u8) -> Result<[u8; FEATURE_BYTES], String> {
    if config.stages.is_empty() || config.stages.len() > DPI_STAGES {
        return Err("DPI 档位数量无效".into());
    }
    if active as usize >= config.stages.len() {
        return Err("DPI 档位编号无效".into());
    }
    let mut out = config.raw;
    out[0] = SET_DPI;
    out[1] = PROFILE;
    out[2] = active;
    out[3] = config.stages.len() as u8;
    for (i, stage) in config.stages.iter().enumerate() {
        for dpi in [stage.x, stage.y] {
            if dpi != 0
                && (!(DPI_MIN..=DPI_MAX).contains(&dpi)
                    || !(dpi - DPI_MIN).is_multiple_of(DPI_STEP))
            {
                return Err(format!(
                    "DPI 必须为 0（未启用档位）或 {DPI_MIN}–{DPI_MAX} 且以 {DPI_STEP} 为步进"
                ));
            }
        }
        let put = |out: &mut [u8; FEATURE_BYTES], off: usize, v: u16| {
            out[off + i * 2..off + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        };
        put(&mut out, X_OFFSET, stage.x);
        put(&mut out, Y_OFFSET, stage.y);
        out[RGB_OFFSET + i * 3..RGB_OFFSET + i * 3 + 3].copy_from_slice(&stage.rgb);
    }
    Ok(out)
}
pub fn patch_options(current: &[u8], edits: &[(usize, u8)]) -> Result<[u8; FEATURE_BYTES], String> {
    if current.len() < FEATURE_BYTES {
        return Err("GearHub 设置块长度不足".into());
    }
    let mut out = [0; FEATURE_BYTES];
    out[8..].copy_from_slice(&current[8..FEATURE_BYTES]);
    out[0] = SET_OPTIONS;
    out[17] = 255;
    out[18] = 8;
    for (offset, value) in edits {
        if *offset >= FEATURE_BYTES {
            return Err("设置偏移超出数据块".into());
        }
        out[*offset] = *value;
    }
    Ok(out)
}
pub fn encode_rate(hz: u16) -> Option<u8> {
    Some(match hz {
        125 => 8,
        250 => 4,
        500 => 2,
        1000 => 1,
        2000 => 132,
        4000 => 130,
        8000 => 129,
        _ => return None,
    })
}
pub fn decode_rate(code: u8) -> Option<u16> {
    POLLING_RATES
        .into_iter()
        .find(|hz| encode_rate(*hz) == Some(code))
}

/// AJ179 PRO's physical Forward/Back positions are reversed from the common
/// GearHub map: Forward=slot 3 and Back=slot 4. Other key bytes stay opaque.
pub const PHYSICAL_SLOTS: [(u16, usize, &str); 6] = [
    (0, 0, "Left"),
    (1, 1, "Right"),
    (2, 2, "Middle"),
    (3, 4, "Back"),
    (4, 3, "Forward"),
    (5, 5, "DPI"),
];
pub const KNOWN_ACTIONS: [([u8; 4], BindingAction); 7] = [
    ([0, 0, 0, 0], BindingAction::Disabled),
    ([1, 0, 0xf0, 0], BindingAction::Mouse(1)),
    ([1, 0, 0xf1, 0], BindingAction::Mouse(2)),
    ([1, 0, 0xf2, 0], BindingAction::Mouse(3)),
    ([1, 0, 0xf3, 0], BindingAction::Mouse(4)),
    ([1, 0, 0xf4, 0], BindingAction::Mouse(5)),
    ([20, 0, 0, 0], BindingAction::DpiCycle),
];
pub fn decode_key_matrix(reply: &[u8]) -> Result<[u8; FEATURE_BYTES], String> {
    verify_reply(reply, GET_KEYS, false)?;
    // GET_KEYMATRIX has no echo byte: command response begins with slot 0.
    Ok(reply[..FEATURE_BYTES].try_into().unwrap())
}
pub fn key_action(raw: [u8; 4]) -> BindingAction {
    KNOWN_ACTIONS
        .iter()
        .find(|(wire, _)| *wire == raw)
        .map(|(_, a)| a.clone())
        .unwrap_or(BindingAction::Unknown(raw.to_vec()))
}
pub fn encode_key_action(action: &BindingAction) -> Result<[u8; 4], String> {
    KNOWN_ACTIONS.iter().find(|(_,known)| known == action).map(|(wire,_)| *wire).ok_or_else(|| "AJ179 PRO 此驱动仅支持左/右/中键、前进/后退、DPI 循环和禁用；键盘、宏等原始键位会保留且拒绝覆盖".into())
}
pub fn key_write(slot: usize, raw: [u8; 4]) -> Result<[u8; FEATURE_BYTES], String> {
    if slot >= 14 {
        return Err("键位编号无效".into());
    }
    let mut out = [0; FEATURE_BYTES];
    out[0] = SET_KEY;
    out[1] = PROFILE;
    out[2] = slot as u8;
    out[8..12].copy_from_slice(&raw);
    Ok(out)
}
pub fn key_from_matrix(matrix: &[u8; FEATURE_BYTES], slot: usize) -> [u8; 4] {
    matrix[slot * 4..slot * 4 + 4].try_into().unwrap()
}

pub fn state_from_replies(
    dpi: &DpiConfig,
    options: &[u8],
    battery: Option<u8>,
    wired: bool,
    identity: String,
) -> State {
    let active = dpi
        .stages
        .get(dpi.active as usize)
        .unwrap_or(&dpi.stages[0]);
    let visible_len = dpi
        .stages
        .iter()
        .rposition(|s| s.x != 0 || s.y != 0)
        .map_or(0, |i| i + 1);
    let visible: Vec<_> = dpi.stages.iter().take(visible_len).map(|s| s.x).collect();
    let rate = options
        .get(RATE_OFFSET)
        .and_then(|raw| decode_rate(*raw))
        .filter(|hz| !wired || *hz <= 1000);
    // The supported links are direct USB and 2.4 GHz. Upstream reads the
    // 2.4 GHz timer on both; Bluetooth is not implemented by this plugin.
    let sleep_offset = SLEEP_24G_OFFSET;
    let sleep = options
        .get(sleep_offset..sleep_offset + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]));
    State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: true,
        percent: battery.filter(|v| *v <= 100),
        charging: None,
        wired,
        dpi: Some(active.x),
        rate,
        sleep_seconds: sleep,
        updated: now(),
        identity,
        dpi_stages: visible,
        active_stage: Some(dpi.active),
        settings_error: if rate.is_none() {
            "回报率字段未知或当前连接不支持该值".into()
        } else {
            String::new()
        },
        ..State::default()
    }
}
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    const DPI_FIXTURE: &str = "d40002080000002b58022003e803b004dc0500000000000058022003e803b004dc05000000000000ff000000ff000000ffbd10e0ffff00ff00ffff00ffffffff";
    const OPT_FIXTURE: &str = "d30000000000002c0001020000000c01010000000000000003030407000000000000000000000000140000001400000000006464010000000000000000000000";
    fn unhex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn aj179_v303_capture_decodes_and_writes_preserve_other_bytes() {
        let dpi = decode_dpi(&unhex(DPI_FIXTURE)).unwrap();
        assert_eq!(dpi.active, 2);
        assert_eq!(dpi.stages.len(), 8);
        assert_eq!((dpi.stages[2].x, dpi.stages[2].y), (1000, 1000));
        assert_eq!(dpi.stages[5].x, 0);
        let option = unhex(OPT_FIXTURE);
        assert_eq!(decode_rate(option[RATE_OFFSET]), Some(1000));
        let changed = patch_options(&option, &[(RATE_OFFSET, 2)]).unwrap();
        for i in 8..64 {
            if ![0, 17, 18, RATE_OFFSET].contains(&i) {
                assert_eq!(changed[i], option[i]);
            }
        }
        let encoded = encode_dpi(&dpi, 3).unwrap();
        assert_eq!(encoded[3], 8);
        assert_eq!(
            &encoded[X_OFFSET..X_OFFSET + 32],
            &dpi.raw[X_OFFSET..X_OFFSET + 32]
        );
    }
    #[test]
    fn key_fixture_reversed_slots_and_known_action_encoding() {
        assert_eq!(PHYSICAL_SLOTS[3].1, 4);
        assert_eq!(PHYSICAL_SLOTS[4].1, 3);
        assert_eq!(
            encode_key_action(&BindingAction::Mouse(3)).unwrap(),
            [1, 0, 0xf2, 0]
        );
        let mut matrix = [0; 64];
        matrix[16..20].copy_from_slice(&[1, 0, 0xf3, 0]);
        assert_eq!(
            key_action(key_from_matrix(&matrix, 4)),
            BindingAction::Mouse(4)
        );
    }
    #[test]
    fn command_checksum_and_rate_limits() {
        assert_eq!(encode_command(&[0xd4, 0]).len(), 64);
        assert_eq!(encode_command(&[0xd4, 0])[7], 0x2b);
        assert_eq!(encode_rate(8000), Some(129));
        assert_eq!(decode_rate(0), None);
    }
    #[test]
    fn saved_v303_capture_is_the_protocol_fixture() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/ajazz-aj179-pro-v303.json"))
                .unwrap();
        assert_eq!(fixture["model"], "AJAZZ AJ179 PRO");
        assert_eq!(
            decode_device_id(&unhex(fixture["replies"]["usbVersion"].as_str().unwrap())).unwrap(),
            1851
        );
        let dpi = decode_dpi(&unhex(fixture["replies"]["dpi"].as_str().unwrap())).unwrap();
        assert_eq!(dpi.active, 2);
        let options = unhex(fixture["replies"]["option0"].as_str().unwrap());
        assert_eq!(decode_rate(options[RATE_OFFSET]), Some(1000));
    }
}
