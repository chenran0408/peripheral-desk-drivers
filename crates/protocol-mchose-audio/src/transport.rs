use super::{Model, Result};
use peripheral_desk::hid::{self, Interface};
use peripheral_hid_transport::WinTransport;
use std::{
    collections::BTreeSet,
    fs::OpenOptions,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::Devices::HumanInterfaceDevice::*;

fn descriptor(path: &str) -> Option<std::fs::File> {
    OpenOptions::new().access_mode(0).open(path).ok()
}
#[allow(non_upper_case_globals)]
fn has_report(iface: &Interface, kind: HIDP_REPORT_TYPE, id: u8) -> bool {
    let Some(file) = descriptor(&iface.path) else {
        return false;
    };
    let size = match kind {
        HidP_Input => iface.input_len,
        HidP_Output => iface.output_len,
        _ => iface.feature_len,
    };
    if size == 0 {
        return false;
    }
    unsafe {
        let mut raw = 0;
        if !HidD_GetPreparsedData(file.as_raw_handle(), &mut raw) {
            return false;
        }
        let mut report = vec![0; usize::from(size)];
        let status = HidP_InitializeReportForID(kind, id, raw, report.as_mut_ptr(), size.into());
        HidD_FreePreparsedData(raw);
        status == HIDP_STATUS_SUCCESS
    }
}
fn product_matches(model: &Model, iface: &Interface) -> bool {
    if iface.vid != model.vid || iface.pid != model.pid {
        return false;
    }
    if iface.product.trim().eq_ignore_ascii_case(model.product) {
        return true;
    }
    if model.pid != 273
        || !iface
            .product
            .trim()
            .eq_ignore_ascii_case("USB COMPOSITE DEVICE")
    {
        return false;
    }
    let Some(file) = descriptor(&iface.path) else {
        return false;
    };
    let mut buf = [0u16; 126];
    (unsafe { HidD_GetIndexedString(file.as_raw_handle(), 5, buf.as_mut_ptr().cast(), 252) })
        && String::from_utf16_lossy(&buf[..buf.iter().position(|v| *v == 0).unwrap_or(buf.len())])
            .trim()
            .eq_ignore_ascii_case(model.product)
}
pub struct Physical {
    pub instance: String,
    collections: Vec<Interface>,
}
impl Physical {
    pub fn find(model: &Model, target: Option<&str>) -> Result<Option<Self>> {
        let all = hid::enumerate()?;
        let ids: BTreeSet<_> = all
            .iter()
            .filter(|i| product_matches(model, i))
            .map(Interface::instance_id)
            .filter(|i| target.is_none_or(|t| t == i))
            .collect();
        if ids.len() > 1 {
            return Err("检测到多台同型号设备，请在设备调节中选择接收器".into());
        }
        let Some(instance) = ids.into_iter().next() else {
            return Ok(None);
        };
        let collections = all
            .into_iter()
            .filter(|i| i.vid == model.vid && i.pid == model.pid && i.instance_id() == instance)
            .collect();
        Ok(Some(Self {
            instance,
            collections,
        }))
    }
    fn collection(&self, kind: HIDP_REPORT_TYPE, id: u8) -> Result<&Interface> {
        let mut list = self.collections.iter().filter(|i| has_report(i, kind, id));
        let item = list
            .next()
            .ok_or_else(|| format!("所选设备没有报告 {id:02X} 对应的接口"))?;
        if list.next().is_some() {
            return Err(format!("同一设备有多个报告 {id:02X} 接口，无法确定通道"));
        }
        Ok(item)
    }
    pub fn usb_rev(&self) -> Option<u16> {
        let file = descriptor(&self.collections.first()?.path)?;
        let mut attrs = HIDD_ATTRIBUTES {
            Size: std::mem::size_of::<HIDD_ATTRIBUTES>() as u32,
            ..Default::default()
        };
        if !unsafe { HidD_GetAttributes(file.as_raw_handle(), &mut attrs) } {
            return None;
        }
        format!("{:04x}", attrs.VersionNumber).parse().ok()
    }
    pub fn blk_receiver_version(&self) -> Result<u32> {
        let cap = self.collection(HidP_Feature, 0xaa)?;
        if cap.feature_len < 64 {
            return Err("接收器版本报告长度不足".into());
        }
        let io = hid::Device::open(&cap.path)?;
        let mut req = vec![0; cap.feature_len as usize];
        req[..3].copy_from_slice(&[0xaa, 1, 1]);
        std::thread::sleep(Duration::from_millis(300));
        io.send(&req)?;
        std::thread::sleep(Duration::from_millis(600));
        let data = io.receive(0xaa, req.len())?;
        // WebHID receiveFeatureReport returns the report ID in the DataView.
        if data.len() < 6 || !data.starts_with(&[0xaa, 1]) || data == req {
            return Err("接收器版本应答无效".into());
        }
        data[2..6]
            .iter()
            .map(ToString::to_string)
            .collect::<String>()
            .parse()
            .map_err(|_| "接收器固件版本无效".into())
    }
    pub fn channel(&self, id: u8) -> Result<Channel> {
        let output = self.collection(HidP_Output, id)?;
        let input = self.collection(HidP_Input, id)?;
        Ok(Channel {
            writer: WinTransport::open(&output.path)?,
            reader: WinTransport::open(&input.path)?,
            input_len: input.input_len as usize,
            output_len: output.output_len as usize,
            id,
        })
    }
}
pub struct Channel {
    writer: WinTransport,
    reader: WinTransport,
    input_len: usize,
    output_len: usize,
    id: u8,
}
impl Channel {
    pub fn send(&mut self, payload: &[u8]) -> Result<()> {
        if payload.len() + 1 > self.output_len {
            return Err("音频报告超过当前接口长度".into());
        }
        let mut report = vec![0; self.output_len];
        report[0] = self.id;
        report[1..1 + payload.len()].copy_from_slice(payload);
        self.writer.write(&report, Duration::from_millis(1000))
    }
    pub fn query<T>(&mut self, payload: &[u8], parse: impl Fn(&[u8]) -> Option<T>) -> Result<T> {
        self.reader.flush()?;
        self.send(payload)?;
        let until = Instant::now() + Duration::from_millis(1100);
        while Instant::now() < until {
            let msg = self.reader.read(
                self.input_len,
                until.saturating_duration_since(Instant::now()),
            )?;
            if msg.is_empty() {
                break;
            }
            if msg[0] == self.id {
                if let Some(value) = parse(&msg[1..]) {
                    return Ok(value);
                }
            }
        }
        Err("设备未返回所选音频设置".into())
    }
}
