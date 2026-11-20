use super::{choice, field, AudioPage, Result};
use serde_json::json;
use sha2::{Digest, Sha256};
use windows::Win32::{
    Devices::FunctionDiscovery::{PKEY_Device_ContainerId, PKEY_Device_FriendlyName},
    Media::Audio::{Endpoints::IAudioEndpointVolume, *},
    System::Com::{
        StructuredStorage::{PropVariantClear, PropVariantToGUID, PropVariantToStringAlloc},
        *,
    },
};

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
pub struct Endpoint {
    pub token: i32,
    pub name: String,
    volume: IAudioEndpointVolume,
}
pub struct Endpoints {
    list: Vec<Endpoint>,
    _apartment: Apartment,
}

fn err(e: windows::core::Error) -> String {
    format!("Windows 音频接口：{e}")
}

fn matches_container(guid: windows::core::GUID, instance: &str) -> bool {
    let mut bytes = Vec::with_capacity(16);
    bytes.extend(guid.data1.to_le_bytes());
    bytes.extend(guid.data2.to_le_bytes());
    bytes.extend(guid.data3.to_le_bytes());
    bytes.extend(guid.data4);
    format!("{:x}", Sha256::digest(bytes)) == instance
}

impl Endpoints {
    pub fn open(instance: &str, capture: bool) -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(err)?;
            let apartment = Apartment;
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(err)?;
            let devices = enumerator
                .EnumAudioEndpoints(
                    if capture { eCapture } else { eRender },
                    DEVICE_STATE_ACTIVE,
                )
                .map_err(err)?;
            let mut list = Vec::new();
            for i in 0..devices.GetCount().map_err(err)? {
                let dev = devices.Item(i).map_err(err)?;
                let store = dev.OpenPropertyStore(STGM_READ).map_err(err)?;
                let Ok(mut container) = store.GetValue(&PKEY_Device_ContainerId) else {
                    continue;
                };
                let id = PropVariantToGUID(&container);
                PropVariantClear(&mut container).map_err(err)?;
                if !id.is_ok_and(|g| matches_container(g, instance)) {
                    continue;
                }
                let mut label = store.GetValue(&PKEY_Device_FriendlyName).map_err(err)?;
                let raw_name = PropVariantToStringAlloc(&label);
                PropVariantClear(&mut label).map_err(err)?;
                let raw_name = raw_name.map_err(err)?;
                let name = raw_name.to_string();
                CoTaskMemFree(Some(raw_name.0.cast()));
                let raw_id = dev.GetId().map_err(err)?;
                let id = raw_id.to_string();
                CoTaskMemFree(Some(raw_id.0.cast()));
                let hash = Sha256::digest(id.map_err(|e| e.to_string())?.as_bytes());
                let token = i32::from_le_bytes(hash[..4].try_into().unwrap()) & i32::MAX;
                let volume = dev.Activate(CLSCTX_ALL, None).map_err(err)?;
                list.push(Endpoint {
                    token,
                    name: name.map_err(|e| e.to_string())?,
                    volume,
                });
            }
            list.sort_by_key(|e| e.token);
            if list.windows(2).any(|p| p[0].token == p[1].token) {
                return Err("音频端点标识冲突，无法区分".into());
            }
            Ok(Self {
                list,
                _apartment: apartment,
            })
        }
    }
    pub fn selected(&self, selector: Option<i32>) -> Result<Option<&Endpoint>> {
        match selector {
            Some(value) => self
                .list
                .iter()
                .find(|e| e.token == value)
                .map(Some)
                .ok_or_else(|| "所选音频端点已断开，请重新读取".into()),
            None => Ok(self.list.first()),
        }
    }
    pub fn page(&self, selector: Option<i32>) -> Result<AudioPage> {
        let Some(e) = self.selected(selector)? else {
            return Ok(AudioPage {
                note: "Windows 没有公开这台设备的活动音频端点。请在系统声音设置中启用对应端点。"
                    .into(),
                ..Default::default()
            });
        };
        let (vol, mute) = e.read()?;
        Ok(AudioPage {
            selector: Some(
                json!({"label":"音频端点","value":e.token,"choices":self.list.iter().map(|x|choice(x.token,&x.name)).collect::<Vec<_>>()}),
            ),
            fields: vec![
                field("volume", "音量", vol, 0, 100, "%"),
                super::toggle("mute", "静音", mute),
            ],
            note: "调整所选设备在 Windows 中的音量与静音。".into(),
            ..Default::default()
        })
    }
}
impl Endpoint {
    fn read(&self) -> Result<(i32, i32)> {
        unsafe {
            Ok((
                (self.volume.GetMasterVolumeLevelScalar().map_err(err)? * 100.0).round() as i32,
                i32::from(self.volume.GetMute().map_err(err)?.as_bool()),
            ))
        }
    }
    pub fn write(&self, id: &str, value: i32) -> Result<()> {
        unsafe {
            match id {
                "volume" => self
                    .volume
                    .SetMasterVolumeLevelScalar(value as f32 / 100.0, std::ptr::null())
                    .map_err(err),
                "mute" => self
                    .volume
                    .SetMute(value != 0, std::ptr::null())
                    .map_err(err),
                _ => Err("音频端点字段无效".into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Endpoints;
    #[test]
    #[ignore = "Reads volume from an attached USB audio device matching the explicitly supplied VID:PID"]
    fn matches_only_selected_windows_container() {
        let ids = std::env::var("PERIPHERAL_DESK_AUDIO_USB_TARGET").unwrap();
        let (vid, pid) = ids.split_once(':').unwrap();
        let ids = (
            u16::from_str_radix(vid, 16).unwrap(),
            u16::from_str_radix(pid, 16).unwrap(),
        );
        let physical = peripheral_desk::hid::enumerate()
            .unwrap()
            .into_iter()
            .find(|i| (i.vid, i.pid) == ids)
            .unwrap();
        let endpoints = Endpoints::open(&physical.instance_id(), false).unwrap();
        let current = endpoints.selected(None).unwrap().unwrap();
        assert!((0..=100).contains(&current.read().unwrap().0));
        assert!(Endpoints::open(&"0".repeat(64), false)
            .unwrap()
            .list
            .is_empty());
    }
}
