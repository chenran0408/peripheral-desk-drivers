//! Read and select existing HID++ 0x8100 profiles and DPI slots.
//! Protocol facts and exact device reports: docs/LOGITECH-ONBOARD.md.
use super::{
    transport::WinTransport, verify_logitech_model, Reader, Result, Transport, ONBOARD_FEATURE,
};
use serde_json::{json, Value};

struct Model {
    receivers: &'static [u16],
    wpid: u16,
    name: &'static str,
}

fn model(key: &str) -> Result<Model> {
    let (receivers, wpid, name): (&[u16], _, _) = match key {
        "logitech-g304" => (&[0xc53f], 0x4074, super::G304_NAME),
        "logitech-g305" => (&[0xc53f], 0x4074, super::G305_NAME),
        "logitech-g502-lightspeed" => (
            &[0xc539, 0xc547],
            0x407f,
            "G502 LIGHTSPEED Wireless Gaming Mouse",
        ),
        "logitech-g502-x-plus" => (&[0xc539, 0xc547], 0x4099, "G502 X PLUS"),
        _ => return Err("该型号未接入 HID++ 板载设置".into()),
    };
    Ok(Model {
        receivers,
        wpid,
        name,
    })
}

fn open(key: &str) -> Result<(Reader<WinTransport>, u8, bool)> {
    let model = model(key)?;
    let mut last = format!("未找到 {} 的 LIGHTSPEED 接收器", model.name);
    for i in peripheral_desk::hid::enumerate()?.into_iter().filter(|i| {
        i.vid == 0x046d
            && model.receivers.contains(&i.pid)
            && i.usage_page == 0xff00
            && i.usage == 2
            && i.input_len == 20
            && i.output_len == 20
    }) {
        let identify = || -> Result<_> {
            let mut reader = Reader(WinTransport::open(&i.path)?);
            let info = reader.feature(0x0003)?;
            let ids = reader.call(info, 0, &[])?;
            let name_feature = reader.feature(0x0005)?;
            let name = reader.name(name_feature)?;
            verify_logitech_model(&ids, &name, model.wpid, model.name)?;
            let (feature, version) = reader.feature_info(ONBOARD_FEATURE)?;
            if version != 0 {
                return Err(format!("此系列板载特性版本 {version} 尚无布局记录"));
            }
            Ok((reader, feature, model.wpid == 0x4074))
        };
        match identify() {
            Ok(session) => return Ok(session),
            Err(e) => last = e,
        }
    }
    Err(last)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Onboard,
    Host,
}
impl Mode {
    fn parse(v: u8) -> Result<Self> {
        match v {
            1 => Ok(Self::Onboard),
            2 => Ok(Self::Host),
            _ => Err(format!("未知板载模式 {v}")),
        }
    }
    fn byte(self) -> u8 {
        match self {
            Self::Onboard => 1,
            Self::Host => 2,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Onboard => "onboard",
            Self::Host => "host",
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Profile {
    id: u16,
    enabled: bool,
    dpi: [u16; 5],
}

#[derive(Debug)]
struct Snapshot {
    mode: Mode,
    active_profile: Option<u16>,
    active_dpi_index: Option<u8>,
    profiles: Vec<Profile>,
}
impl Snapshot {
    fn json(&self) -> Value {
        let profiles: Vec<_> = self
            .profiles
            .iter()
            .map(|p| json!({"id":p.id,"enabled":p.enabled,"dpi":p.dpi}))
            .collect();
        json!({"mode":self.mode.label(),"active_profile":self.active_profile,"active_dpi_index":self.active_dpi_index,"profiles":profiles})
    }
}

#[derive(Clone, Copy)]
enum Change {
    Mode(Mode),
    Profile(u16),
    Dpi { profile: u16, index: u8 },
}

struct Board<T> {
    reader: Reader<T>,
    feature: u8,
    legacy_empty_sector: bool,
}
impl<T: Transport> Board<T> {
    fn call(&mut self, function: u8, args: &[u8]) -> Result<Vec<u8>> {
        self.reader.call(self.feature, function, args)
    }
    fn chunk(&mut self, sector: u16, offset: u16) -> Result<[u8; 16]> {
        let [s, t] = sector.to_be_bytes();
        let [i, j] = offset.to_be_bytes();
        self.call(0x50, &[s, t, i, j])?
            .try_into()
            .map_err(|_| "板载存储读取未返回 16 字节".into())
    }
    fn mode(&mut self) -> Result<Mode> {
        Mode::parse(*self.call(0x20, &[])?.first().ok_or("模式回复为空")?)
    }
    fn active_profile(&mut self) -> Result<u16> {
        let r = self.call(0x40, &[])?;
        Ok(u16::from_be_bytes(
            r.get(..2).ok_or("当前配置回复不完整")?.try_into().unwrap(),
        ))
    }
    fn active_index(&mut self) -> Result<u8> {
        self.call(0xb0, &[])?
            .first()
            .copied()
            .filter(|v| *v < 5)
            .ok_or_else(|| "设备未报告有效 DPI 档位".into())
    }
    fn profiles(&mut self) -> Result<Vec<Profile>> {
        let descriptor = self.call(0, &[])?;
        if descriptor.len() < 9 || descriptor[0] != 1 || !(1..=5).contains(&descriptor[1]) {
            return Err("此固件的板载配置布局尚无记录".into());
        }
        let count = usize::from(descriptor[3]);
        let size = u16::from_be_bytes([descriptor[7], descriptor[8]]);
        if size < 16 || 4 * count > usize::from(size - 2) {
            return Err("设备报告的配置数量与存储布局不符".into());
        }
        let mut directory = 0;
        let mut block = match self.chunk(0, 0) {
            Ok(b) if b[..4] != [0; 4] && b[..4] != [0xff; 4] => b,
            Ok(_) => {
                directory = 0x100;
                self.chunk(directory, 0)?
            }
            // First-generation G304/G305 can reject a never-written user sector.
            Err(e) if self.legacy_empty_sector && e.contains("错误码 0x02") => {
                directory = 0x100;
                self.chunk(directory, 0)?
            }
            Err(e) => return Err(e),
        };
        let mut profiles = Vec::new();
        let mut block_offset = 0;
        for slot in 0..count {
            let position = slot as u16 * 4;
            if position >= block_offset + 16 {
                block_offset = position.min(size - 16);
                block = self.chunk(directory, block_offset)?;
            }
            let i = usize::from(position - block_offset);
            let id = u16::from_be_bytes([block[i], block[i + 1]]);
            if id == u16::MAX {
                break;
            }
            if id == 0 || id == 0x100 || profiles.iter().any(|p: &Profile| p.id == id) {
                return Err("设备返回了无效或重复的配置槽".into());
            }
            let enabled = block[i + 2] != 0;
            // Unused slots can contain erased memory. Only enabled slots are selectable.
            let mut dpi = [0; 5];
            if enabled {
                let head = self.chunk(id, 0)?;
                for (value, pair) in dpi.iter_mut().zip(head[3..13].as_chunks::<2>().0) {
                    *value = u16::from_le_bytes(*pair);
                }
            }
            profiles.push(Profile { id, enabled, dpi });
        }
        Ok(profiles)
    }
    fn read(&mut self) -> Result<Snapshot> {
        let mode = self.mode()?;
        if mode == Mode::Host {
            return Ok(Snapshot {
                mode,
                active_profile: None,
                active_dpi_index: None,
                profiles: Vec::new(),
            });
        }
        let active_profile = self.active_profile()?;
        let profiles = self.profiles()?;
        let active_dpi_index = self.active_index().ok();
        Ok(Snapshot {
            mode,
            active_profile: Some(active_profile),
            active_dpi_index,
            profiles,
        })
    }
    fn write(&mut self, change: Change) -> Result<()> {
        match change {
            Change::Mode(m) => self.call(0x10, &[m.byte()]),
            Change::Profile(id) => self.call(0x30, &id.to_be_bytes()),
            Change::Dpi { index, .. } => self.call(0xc0, &[index]),
        }
        .map(|_| ())
    }
    fn restore(&mut self, before: &Snapshot) -> Result<()> {
        if self.mode().ok() != Some(before.mode) {
            self.write(Change::Mode(before.mode))?;
            if self.mode()? != before.mode {
                return Err("恢复后的运行模式不匹配".into());
            }
        }
        if let Some(profile) = before.active_profile {
            if self.active_profile().ok() != Some(profile) {
                self.write(Change::Profile(profile))?;
                if self.active_profile()? != profile {
                    return Err("恢复后的配置槽不匹配".into());
                }
            }
            if let Some(index) = before.active_dpi_index {
                if self.active_index().ok() != Some(index) {
                    self.write(Change::Dpi { profile, index })?;
                    if self.active_index()? != index {
                        return Err("恢复后的 DPI 档位不匹配".into());
                    }
                }
            }
        }
        Ok(())
    }
    fn apply(&mut self, change: Change) -> Result<Value> {
        let before = self.read()?;
        let already = match change {
            Change::Mode(m) => before.mode == m,
            Change::Profile(id) => {
                if before.mode != Mode::Onboard
                    || !before.profiles.iter().any(|p| p.id == id && p.enabled)
                {
                    return Err("请在板载模式选择已启用的配置槽".into());
                }
                before.active_profile == Some(id)
            }
            Change::Dpi { profile, index } => {
                if before.active_profile != Some(profile) {
                    return Err("当前板载配置已变化，请刷新后再选择 DPI 档位".into());
                }
                let dpi = before
                    .profiles
                    .iter()
                    .find(|p| p.id == profile && p.enabled)
                    .and_then(|p| p.dpi.get(usize::from(index)))
                    .copied();
                if matches!(dpi, None | Some(0 | 0xffff)) || before.active_dpi_index.is_none() {
                    return Err("该 DPI 档位没有有效值，或设备未报告当前档位".into());
                }
                before.active_dpi_index == Some(index)
            }
        };
        if already {
            return Ok(before.json());
        }
        let result = self
            .write(change)
            .and_then(|()| self.read())
            .and_then(|after| {
                let verified = match change {
                    Change::Mode(m) => after.mode == m,
                    Change::Profile(id) => after.active_profile == Some(id),
                    Change::Dpi { profile, index } => {
                        after.active_profile == Some(profile)
                            && after.active_dpi_index == Some(index)
                    }
                };
                if verified {
                    Ok(after.json())
                } else {
                    Err("设备读回的板载选择与请求不符".into())
                }
            });
        result.map_err(|err| match self.restore(&before) {
            Ok(()) => format!("设置失败，已核对原模式及已知原档位：{err}"),
            Err(back) => format!("设置失败：{err}；恢复未能确认：{back}"),
        })
    }
}

pub fn handles(op: &str) -> bool {
    matches!(
        op,
        "read-onboard-profiles"
            | "set-onboard-mode"
            | "select-onboard-profile"
            | "select-onboard-dpi"
    )
}
pub fn handle(key: &str, request: &Value) -> Result<Value> {
    let profile = || {
        request["profile"]
            .as_u64()
            .and_then(|id| u16::try_from(id).ok())
            .ok_or("请指定板载配置槽")
    };
    let change = match request["op"].as_str() {
        Some("read-onboard-profiles") => None,
        Some("set-onboard-mode") => Some(Change::Mode(match request["mode"].as_str() {
            Some("host") => Mode::Host,
            Some("onboard") => Mode::Onboard,
            _ => return Err("请选择板载或主机模式".into()),
        })),
        Some("select-onboard-profile") => Some(Change::Profile(profile()?)),
        Some("select-onboard-dpi") => Some(Change::Dpi {
            profile: profile()?,
            index: request["index"]
                .as_u64()
                .and_then(|v| u8::try_from(v).ok())
                .ok_or("请输入 DPI 档位索引")?,
        }),
        _ => return Err("未识别的板载设置操作".into()),
    };
    let (reader, feature, legacy_empty_sector) = open(key)?;
    let mut board = Board {
        reader,
        feature,
        legacy_empty_sector,
    };
    match change {
        None => Ok(board.read()?.json()),
        Some(c) => board.apply(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Mouse {
        mode: u8,
        profile: u16,
        index: u8,
        rom_only: bool,
        reject_next_profile: bool,
        writes: Vec<(u8, u16)>,
    }
    impl Transport for Mouse {
        fn exchange(&mut self, request: &[u8; 20]) -> Result<Vec<u8>> {
            let function = request[3] & 0xf0;
            assert_eq!(request[..3], [0x11, 1, 9]);
            let arg = u16::from_be_bytes([request[4], request[5]]);
            let mut out = [0; 16];
            match function {
                0x00 => {
                    out[..9].copy_from_slice(&[1, 3, 1, 2, 2, 11, 8, 1, 0]);
                }
                0x20 => out[0] = self.mode,
                0x40 => out[..2].copy_from_slice(&self.profile.to_be_bytes()),
                0xb0 => out[0] = self.index,
                0x50 => {
                    assert_eq!(request[6..8], [0, 0]);
                    match arg {
                        0 if self.rom_only => return Err("HID++ 查询被拒绝，错误码 0x02".into()),
                        0 | 0x100 => {
                            let bank = (arg >> 8) as u8;
                            out[..12]
                                .copy_from_slice(&[bank, 1, 1, 0, bank, 2, 1, 0, 255, 255, 0, 0]);
                        }
                        1 | 2 | 0x101 | 0x102 => {
                            for (i, dpi) in [400u16, 800, 0, 1600, 0].iter().enumerate() {
                                out[3 + i * 2..5 + i * 2].copy_from_slice(&dpi.to_le_bytes());
                            }
                        }
                        _ => panic!("unexpected sector {arg}"),
                    }
                }
                0x10 => {
                    self.writes.push((function, u16::from(request[4])));
                    self.mode = request[4];
                }
                0x30 => {
                    self.writes.push((function, arg));
                    self.profile = arg;
                    self.index = 0;
                    if std::mem::take(&mut self.reject_next_profile) {
                        return Err("模拟确认包丢失".into());
                    }
                }
                0xc0 => {
                    self.writes.push((function, u16::from(request[4])));
                    self.index = request[4];
                }
                _ => panic!("unexpected command {function:#x}"),
            }
            Ok(out.to_vec())
        }
    }
    fn board() -> Board<Mouse> {
        Board {
            reader: Reader(Mouse {
                mode: 1,
                profile: 1,
                index: 1,
                rom_only: false,
                reject_next_profile: false,
                writes: vec![],
            }),
            feature: 9,
            legacy_empty_sector: true,
        }
    }
    #[test]
    fn onboard_profiles_read_le_dpi_keep_empty_slots_and_select_without_writing_memory() {
        let mut board = board();
        assert_eq!(
            board.read().unwrap().json()["profiles"][0]["dpi"],
            json!([400, 800, 0, 1600, 0])
        );
        assert!(board
            .apply(Change::Dpi {
                profile: 1,
                index: 2
            })
            .is_err());
        let selected = board.apply(Change::Profile(2)).unwrap();
        assert_eq!(selected["active_profile"], 2);
        assert!(board
            .apply(Change::Dpi {
                profile: 1,
                index: 3
            })
            .is_err());
        assert_eq!(
            board
                .apply(Change::Dpi {
                    profile: 2,
                    index: 3
                })
                .unwrap()["active_dpi_index"],
            3
        );
        assert_eq!(board.reader.0.writes, vec![(0x30, 2), (0xc0, 3)]);
    }
    #[test]
    fn onboard_failed_selection_restores_the_previous_profile_and_dpi() {
        let mut board = board();
        board.reader.0.reject_next_profile = true;
        assert!(board
            .apply(Change::Profile(2))
            .unwrap_err()
            .contains("已核对"));
        assert_eq!((board.reader.0.profile, board.reader.0.index), (1, 1));
        assert_eq!(board.reader.0.writes, vec![(0x30, 2), (0x30, 1), (0xc0, 1)]);
    }
    #[test]
    fn onboard_host_mode_is_explicit_and_factory_sectors_keep_their_high_byte() {
        let mut board = board();
        board.reader.0.mode = 2;
        board.reader.0.rom_only = true;
        board.reader.0.profile = 0x101;
        assert_eq!(board.read().unwrap().json()["profiles"], json!([]));
        assert!(board.reader.0.writes.is_empty());
        let result = board.apply(Change::Mode(Mode::Onboard)).unwrap();
        assert_eq!(result["profiles"][0]["id"], 0x101);
        let selected = board.apply(Change::Profile(0x102)).unwrap();
        assert_eq!(selected["active_profile"], 0x102);
        assert_eq!(board.reader.0.writes, vec![(0x10, 1), (0x30, 0x102)]);
    }
}
