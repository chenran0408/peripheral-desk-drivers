//! MCHOSE MagDock lighting codec and guarded HID adapter.
//!
//! Protocol facts are adapted from OpenMouse Project `mouse-protocol`, commit
//! `bac031567b28f2a55c616fc525b03f2d5fa452cc`, `src/mchose/dock.ts`,
//! `src/drivers/mchose/dock-hid.ts`, and its real-device records in
//! `docs/mchose-protocol.md`. The upstream repository is AGPL-3.0-or-later.
//! This Rust implementation is an independent port; no upstream code is copied.
//!
//! The upstream WebHID API sends a 63-byte unnumbered report with report ID 0.
//! Windows HID read/write buffers include that report ID byte, so this adapter
//! matches 64-byte input/output reports and prefixes/strips the zero byte.

use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant};

const VID: u16 = 0x3837;
const PID: u16 = 0x1012;
const USAGE_PAGE: u16 = 0xff00;
const USAGE: u16 = 0x0001;
const WINDOWS_REPORT_LEN: usize = 64;
const FRAME_LEN: usize = 63;
const HEADER_LEN: usize = 6;
const READ_COMMAND: u8 = 7;
const WRITE_COMMAND: u8 = 39;
const REPLY_TIMEOUT: Duration = Duration::from_millis(700);
const WRITE_SETTLE: Duration = Duration::from_millis(400);

pub const DEVICE_KEY: &str = "mchose-magdock";
pub type Result<T> = std::result::Result<T, String>;

/// Fields exposed by the confirmed command-39 write. `effect_count` and
/// `direction` are copied from a fresh read when applying a partial UI edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lighting {
    pub enabled: bool,
    pub effect: u8,
    pub effect_count: u8,
    pub speed: u8,
    pub brightness: u8,
    pub music_sync: bool,
    pub rgb: [u8; 3],
    pub direction: u8,
}

/// Read payload is 30 bytes; the firmware reply contains reserved/echo bytes
/// between the ten write parameters and byte 29. Keep them available to callers
/// and tests, while never inventing write semantics for them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadBlock {
    pub lighting: Lighting,
    pub payload: [u8; 30],
}

pub fn encode_frame(command: u8, params: &[u8]) -> Result<[u8; FRAME_LEN]> {
    if params.len() > FRAME_LEN - HEADER_LEN || params.len() > u8::MAX as usize {
        return Err("MagDock 请求参数过长".into());
    }
    let mut frame = [0u8; FRAME_LEN];
    frame[0] = 0xaa;
    frame[1] = command;
    frame[2] = 0;
    frame[3] = 0;
    frame[4] = 0;
    frame[5] = params.len() as u8;
    frame[HEADER_LEN..HEADER_LEN + params.len()].copy_from_slice(params);
    Ok(frame)
}

pub fn decode_reply(wire: &[u8], command: u8) -> Result<&[u8]> {
    if wire.len() != FRAME_LEN {
        return Err(format!(
            "MagDock 回复长度不匹配：期望 {FRAME_LEN}，收到 {}",
            wire.len()
        ));
    }
    if wire[0] != 0xaa || wire[1] != command || !matches!(wire[2], 0 | 2) {
        return Err("MagDock 回复帧头、命令或类型不匹配".into());
    }
    let len = wire[5] as usize;
    if HEADER_LEN + len > wire.len() {
        return Err("MagDock 回复参数长度越界".into());
    }
    Ok(&wire[HEADER_LEN..HEADER_LEN + len])
}

pub fn decode_lighting(payload: &[u8]) -> Result<ReadBlock> {
    if payload.len() < 30 {
        return Err(format!(
            "MagDock 灯效回复不完整：期望至少 30 字节，收到 {}",
            payload.len()
        ));
    }
    let mut raw = [0u8; 30];
    raw.copy_from_slice(&payload[..30]);
    if !matches!(raw[0], 0 | 1) || !matches!(raw[5], 0 | 1) {
        return Err("MagDock 开关或 Music 同步值不是 0/1；为避免覆盖未知原值，已停止".into());
    }
    Ok(ReadBlock {
        lighting: Lighting {
            enabled: raw[0] == 1,
            effect: raw[1],
            effect_count: raw[2],
            speed: raw[3],
            brightness: raw[4],
            music_sync: raw[5] == 1,
            rgb: [raw[6], raw[7], raw[8]],
            // The read reply reports direction at byte 29; command 39 writes
            // that value in parameter slot 9, as the upstream implementation.
            direction: raw[29],
        },
        payload: raw,
    })
}

pub fn encode_lighting(state: &Lighting) -> Result<[u8; FRAME_LEN]> {
    if state.effect > 5 {
        return Err("MagDock 灯效编号必须为 0–5".into());
    }
    if state.speed > 4 || state.brightness > 4 {
        return Err("MagDock 速度和亮度必须为 0–4".into());
    }
    encode_frame(
        WRITE_COMMAND,
        &[
            u8::from(state.enabled),
            state.effect,
            state.effect_count,
            state.speed,
            state.brightness,
            u8::from(state.music_sync),
            state.rgb[0],
            state.rgb[1],
            state.rgb[2],
            state.direction,
        ],
    )
}

fn matching_interface(i: &peripheral_desk::hid::Interface) -> bool {
    i.vid == VID
        && i.pid == PID
        && i.usage_page == USAGE_PAGE
        && i.usage == USAGE
        && i.input_len as usize == WINDOWS_REPORT_LEN
        && i.output_len as usize == WINDOWS_REPORT_LEN
}

fn open_unique() -> Result<WinTransport> {
    let interfaces = peripheral_desk::hid::enumerate()?;
    let mut matching = interfaces.iter().filter(|i| matching_interface(i));
    let Some(interface) = matching.next() else {
        return Err("未找到精确的 MCHOSE MagDock 3837:1012 / FF00:0001 HID 集合".into());
    };
    if matching.next().is_some() {
        return Err("找到多个 MCHOSE MagDock HID 集合；为避免写入错误设备已停止".into());
    }
    WinTransport::open(&interface.path)
}

fn send(transport: &mut WinTransport, frame: &[u8; FRAME_LEN]) -> Result<()> {
    let mut wire = [0u8; WINDOWS_REPORT_LEN];
    wire[1..].copy_from_slice(frame);
    transport.write(&wire, Duration::from_millis(250))
}

fn request(transport: &mut WinTransport, command: u8) -> Result<Vec<u8>> {
    let frame = encode_frame(command, &[])?;
    let mut last_error = String::new();
    for _ in 0..6 {
        if let Err(error) = transport.flush() {
            last_error = error;
            continue;
        }
        if let Err(error) = send(transport, &frame) {
            last_error = error;
            continue;
        }
        let attempt_deadline = Instant::now() + REPLY_TIMEOUT;
        while Instant::now() < attempt_deadline {
            let remaining = attempt_deadline.saturating_duration_since(Instant::now());
            let wire = transport.read(WINDOWS_REPORT_LEN, remaining)?;
            if wire.is_empty() {
                break;
            }
            if wire.len() != WINDOWS_REPORT_LEN || wire[0] != 0 {
                last_error = "MagDock Windows HID report ID 或长度不匹配".into();
                continue;
            }
            match decode_reply(&wire[1..], command) {
                Ok(payload) => return Ok(payload.to_vec()),
                Err(error) => last_error = error,
            }
        }
    }
    if last_error.is_empty() {
        Err(format!("MagDock 命令 {command} 在重试后仍未回复"))
    } else {
        Err(format!("MagDock 命令 {command} 无有效回复：{last_error}"))
    }
}

fn read_block(transport: &mut WinTransport) -> Result<ReadBlock> {
    decode_lighting(&request(transport, READ_COMMAND)?)
}

pub fn read_lighting() -> Result<ReadBlock> {
    read_block(&mut open_unique()?)
}

/// Apply the full confirmed lighting block. If write verification fails,
/// attempt to restore and verify the exact original ten writable fields.
pub fn read_state() -> Result<peripheral_desk::model::State> {
    let block = read_lighting()?;
    Ok(peripheral_desk::model::State {
        key: DEVICE_KEY.into(),
        present: true,
        fresh: true,
        wired: true,
        light: Some(if block.lighting.enabled {
            block.lighting.effect + 1
        } else {
            0
        }),
        updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        identity: "MCHOSE MagDock USB".into(),
        ..Default::default()
    })
}
pub fn apply_lighting(desired: &Lighting) -> Result<ReadBlock> {
    update_lighting(|original| {
        let mut next = desired.clone();
        next.effect_count = original.effect_count;
        next.direction = original.direction;
        next.music_sync = next.effect == 5;
        if next.effect > 1 {
            next.rgb = original.rgb;
        }
        next
    })
}
fn update_lighting(change: impl FnOnce(&Lighting) -> Lighting) -> Result<ReadBlock> {
    let mut transport = open_unique()?;
    let original = read_block(&mut transport)?;
    // Verify that the baseline can be restored before any mutation.
    let restore_frame = encode_lighting(&original.lighting)?;
    let next = change(&original.lighting);
    let frame = encode_lighting(&next)?;
    let result = send(&mut transport, &frame)
        .and_then(|_| {
            std::thread::sleep(WRITE_SETTLE);
            read_block(&mut transport)
        })
        .and_then(|readback| {
            if writable_eq(&readback.lighting, &next) {
                Ok(readback)
            } else {
                Err("MagDock 灯效写入后读回不一致".into())
            }
        });
    match result {
        Ok(readback) => Ok(readback),
        Err(error) => {
            let restore = send(&mut transport, &restore_frame).and_then(|_| {
                std::thread::sleep(WRITE_SETTLE);
                let restored = read_block(&mut transport)?;
                if writable_eq(&restored.lighting, &original.lighting) {
                    Ok(())
                } else {
                    Err("恢复后的 MagDock 灯效读回不一致".into())
                }
            });
            match restore {
                Ok(()) => Err(format!("{error}；已恢复并核对原灯效")),
                Err(restore_error) => Err(format!("{error}；尝试恢复原灯效失败：{restore_error}")),
            }
        }
    }
}

pub fn switch_lighting(enabled: bool) -> Result<ReadBlock> {
    update_lighting(|original| {
        let mut state = original.clone();
        state.enabled = enabled;
        state
    })
}

fn writable_eq(actual: &Lighting, expected: &Lighting) -> bool {
    actual.enabled == expected.enabled
        && actual.effect == expected.effect
        && actual.effect_count == expected.effect_count
        && actual.speed == expected.speed
        && actual.brightness == expected.brightness
        && actual.music_sync == expected.music_sync
        && actual.rgb == expected.rgb
        && actual.direction == expected.direction
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPTURED: [u8; 30] = [
        1, 3, 6, 2, 2, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn captured_report_decodes_and_round_trips_the_verified_ten_bytes() {
        let block = decode_lighting(&CAPTURED).unwrap();
        assert_eq!(block.lighting.effect, 3);
        assert_eq!(block.lighting.effect_count, 6);
        assert_eq!(block.lighting.rgb, [255, 0, 0]);
        let frame = encode_lighting(&block.lighting).unwrap();
        assert_eq!(frame[5], 10);
        assert_eq!(&frame[6..16], &CAPTURED[..10]);
    }

    #[test]
    fn parameter_edit_preserves_effect_count_direction_and_reserved_readback() {
        let mut payload = CAPTURED;
        payload[2] = 7;
        payload[12] = 0xa5;
        payload[29] = 4;
        let original = decode_lighting(&payload).unwrap();
        let mut next = original.lighting.clone();
        next.brightness = 4;
        next.effect_count = original.lighting.effect_count;
        next.direction = original.lighting.direction;
        let frame = encode_lighting(&next).unwrap();
        assert_eq!(frame[8], 7);
        assert_eq!(frame[15], 4);
        assert_eq!(
            original.payload[12], 0xa5,
            "reserved readback remains captured"
        );
    }

    #[test]
    fn frames_validate_header_command_and_payload_size() {
        let request = encode_frame(READ_COMMAND, &[]).unwrap();
        assert_eq!(request[..6], [0xaa, 7, 0, 0, 0, 0]);
        let mut reply = request;
        reply[2] = 2;
        reply[5] = 30;
        assert!(decode_reply(&reply, READ_COMMAND).is_ok());
        assert!(decode_reply(&reply, WRITE_COMMAND).is_err());
        reply[5] = 255;
        assert!(decode_reply(&reply, READ_COMMAND).is_err());
    }

    #[test]
    fn invalid_ranges_are_rejected_before_write() {
        let mut state = decode_lighting(&CAPTURED).unwrap().lighting;
        state.speed = 5;
        assert!(encode_lighting(&state).is_err());
        state.speed = 4;
        state.effect = 6;
        assert!(encode_lighting(&state).is_err());
    }

    #[test]
    fn unknown_boolean_encoding_is_rejected_instead_of_being_rewritten_as_false() {
        let mut payload = CAPTURED;
        payload[5] = 2;
        assert!(decode_lighting(&payload).is_err());
    }
}
