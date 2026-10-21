use peripheral_desk::{
    controls::{Binding, BindingAction, ControlAction, ControlState},
    hid::{self, Device, Interface},
    model::{Action, State},
};
use peripheral_desk_protocol_ajazz_aj179_pro as p;
use serde_json::{json, Value};
use std::{thread, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Link {
    Usb,
    Receiver,
}
struct Mouse {
    device: Device,
    interface: Interface,
    link: Link,
    selected: bool,
    device_id: u32,
}

fn matches(i: &Interface) -> bool {
    i.vid == p::VID
        && [p::USB_PID, p::RECEIVER_PID].contains(&i.pid)
        && i.usage_page == 0xffff
        && i.usage == 2
        && i.feature_len == p::WINDOWS_FEATURE_BYTES as u16
}
fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
impl Mouse {
    fn open() -> Result<Self, String> {
        let found: Vec<_> = hid::enumerate()?.into_iter().filter(matches).collect();
        let mut last =
            "未发现 AJAZZ AJ179 PRO USB 或 2.4 GHz 接收器接口（3151:4026/402D）".to_string();
        for interface in found {
            match Device::open(&interface.path).and_then(|device| {
                let mut mouse = Self {
                    device,
                    link: if interface.pid == p::RECEIVER_PID {
                        Link::Receiver
                    } else {
                        Link::Usb
                    },
                    interface,
                    selected: false,
                    device_id: 0,
                };
                let reply = mouse.exchange(p::get_request(p::GET_DEVICE_ID).to_vec(), true)?;
                mouse.device_id = p::decode_device_id(&reply)?;
                if mouse.device_id != 1851 {
                    return Err(format!(
                        "HID 设备 id {} 不是 AJ179 PRO（1851）",
                        mouse.device_id
                    ));
                }
                Ok(mouse)
            }) {
                Ok(mouse) => return Ok(mouse),
                Err(e) => last = e,
            }
        }
        Err(last)
    }
    fn raw_send(&self, wire: &[u8]) -> Result<(), String> {
        // Windows HID buffers include report id 0; GearHub's unnumbered
        // feature payload follows it as 64 bytes.
        let mut report = [0; p::WINDOWS_FEATURE_BYTES];
        let len = wire.len().min(p::FEATURE_BYTES);
        report[1..1 + len].copy_from_slice(&wire[..len]);
        self.device.send(&report)
    }
    fn raw_read(&self) -> Result<Vec<u8>, String> {
        let report = self.device.receive(0, p::WINDOWS_FEATURE_BYTES)?;
        if report.len() != p::WINDOWS_FEATURE_BYTES || report[0] != 0 {
            return Err("AJ179 PRO Windows Feature report id/length 不匹配".into());
        }
        Ok(report[1..].to_vec())
    }
    fn status(&self) -> Result<Vec<u8>, String> {
        self.raw_send(&[p::DONGLE_STATUS])?;
        pause(10);
        self.raw_read()
    }
    fn select_mouse(&mut self) -> Result<(), String> {
        if self.selected {
            return Ok(());
        }
        self.raw_send(&p::DONGLE_SELECT)?;
        pause(50);
        self.selected = true;
        Ok(())
    }
    fn wait_ready(&self, send: bool) -> Result<(), String> {
        for _ in 0..8 {
            pause(100);
            let s = self.status()?;
            if s.len() >= 6 && s[if send { 5 } else { 0 }] == 1 {
                return Ok(());
            }
        }
        Err(if send {
            "AJ179 PRO 接收器未就绪，无法发送"
        } else {
            "AJ179 PRO 接收器未就绪，无法读取"
        }
        .into())
    }
    fn exchange(&mut self, request: Vec<u8>, echoes: bool) -> Result<Vec<u8>, String> {
        let encoded = p::encode_command(&request);
        match self.link {
            Link::Usb => {
                self.raw_send(&encoded)?;
                pause(10);
                let reply = self.raw_read()?;
                p::verify_reply(&reply, request[0], echoes)?;
                Ok(reply)
            }
            Link::Receiver => {
                self.select_mouse()?;
                self.wait_ready(true)?;
                self.raw_send(&encoded)?;
                self.wait_ready(false)?;
                self.raw_send(&[p::DONGLE_NOTICE_READ])?;
                pause(10);
                let reply = self.raw_read()?;
                p::verify_reply(&reply, request[0], echoes)?;
                Ok(reply)
            }
        }
    }
    fn write(&mut self, request: Vec<u8>) -> Result<(), String> {
        let encoded = p::encode_command(&request);
        match self.link {
            Link::Usb => {
                self.raw_send(&encoded)?;
                pause(10);
            }
            Link::Receiver => {
                self.select_mouse()?;
                self.wait_ready(true)?;
                self.raw_send(&encoded)?;
                pause(50);
            }
        }
        Ok(())
    }
    fn options(&mut self) -> Result<Vec<u8>, String> {
        self.exchange(p::get_request(p::GET_OPTIONS).to_vec(), true)
    }
    fn dpi(&mut self) -> Result<p::DpiConfig, String> {
        let reply = self.exchange(p::get_request(p::GET_DPI).to_vec(), true)?;
        p::decode_dpi(&reply)
    }
    fn keys(&mut self) -> Result<[u8; 64], String> {
        let reply = self.exchange(vec![p::GET_KEYS, p::PROFILE], false)?;
        p::decode_key_matrix(&reply)
    }
    fn battery(&mut self) -> Option<u8> {
        if self.link == Link::Usb {
            return None;
        }
        self.status().ok().and_then(|s| {
            if s.len() >= 6 && s[4] == 0 && (1..=100).contains(&s[2]) {
                Some(s[2])
            } else {
                None
            }
        })
    }
    fn identity(&self) -> String {
        format!(
            "AJAZZ AJ179 PRO · {} · firmware device id 1851 ({:04X}:{:04X})",
            if self.link == Link::Usb {
                "USB"
            } else {
                "2.4 GHz 接收器"
            },
            self.interface.vid,
            self.interface.pid
        )
    }
    fn read_state(&mut self) -> Result<State, String> {
        let options = self.options()?;
        let dpi = self.dpi()?;
        let battery = self.battery();
        Ok(p::state_from_replies(
            &dpi,
            &options,
            battery,
            self.link == Link::Usb,
            self.identity(),
        ))
    }
    fn verify_options(&mut self, expected: &[u8; 64]) -> Result<(), String> {
        let actual = self.options()?;
        for i in 8..64 {
            if ![17, 18].contains(&i) && actual[i] != expected[i] {
                return Err(format!("设置读回不一致（字节 {i}）"));
            }
        }
        Ok(())
    }
    fn restore_options(&mut self, original: &[u8; 64], error: String) -> Result<(), String> {
        let restore = p::patch_options(original, &[])?;
        match self
            .write(restore.to_vec())
            .and_then(|_| self.verify_options(original))
        {
            Ok(()) => Err(format!("{error}；已恢复并核对原设置")),
            Err(e) => Err(format!("{error}；恢复原设置失败：{e}")),
        }
    }
    fn verify_dpi(&mut self, expected: &p::DpiConfig, active: u8) -> Result<(), String> {
        let actual = self.dpi()?;
        if actual.active != active || actual.stages != expected.stages {
            return Err("DPI 档位或颜色读回不一致".into());
        }
        Ok(())
    }
    fn restore_dpi(&mut self, original: &p::DpiConfig, error: String) -> Result<(), String> {
        let bytes = p::encode_dpi(original, original.active)?;
        match self
            .write(bytes.to_vec())
            .and_then(|_| self.verify_dpi(original, original.active))
        {
            Ok(()) => Err(format!("{error}；已恢复并核对原 DPI 表")),
            Err(e) => Err(format!("{error}；恢复原 DPI 表失败：{e}")),
        }
    }
    fn apply(&mut self, action: Action) -> Result<(), String> {
        match action {
            Action::Dpi(value) => {
                let original = self.dpi()?;
                if !(p::DPI_MIN..=p::DPI_MAX).contains(&value)
                    || !(value - p::DPI_MIN).is_multiple_of(p::DPI_STEP)
                {
                    return Err("DPI 必须为 50–26000 且以 50 为步进".into());
                }
                let mut expected = original.clone();
                let active = expected.active as usize;
                if expected.stages[active].x == 0 || expected.stages[active].y == 0 {
                    return Err("当前 DPI 档位未启用，已拒绝写入".into());
                }
                expected.stages[active].x = value;
                expected.stages[active].y = value;
                let bytes = p::encode_dpi(&expected, expected.active)?;
                if let Err(e) = self
                    .write(bytes.to_vec())
                    .and_then(|_| self.verify_dpi(&expected, expected.active))
                {
                    return self.restore_dpi(&original, e);
                }
                Ok(())
            }
            Action::Stage(index) => {
                let original = self.dpi()?;
                let active = index;
                let stage = original
                    .stages
                    .get(active as usize)
                    .ok_or("DPI 档位编号无效")?;
                if stage.x == 0 || stage.y == 0 {
                    return Err("AJ179 PRO 未启用该 DPI 档位".into());
                }
                let bytes = p::encode_dpi(&original, active)?;
                if let Err(e) = self
                    .write(bytes.to_vec())
                    .and_then(|_| self.verify_dpi(&original, active))
                {
                    return self.restore_dpi(&original, e);
                }
                Ok(())
            }
            Action::Rate(hz) => {
                if self.link == Link::Usb && hz > 1000 {
                    return Err("USB 有线模式最高支持 1000 Hz".into());
                }
                let code = p::encode_rate(hz).ok_or("AJ179 PRO 不支持该回报率")?;
                let original = self.options()?;
                let expected = p::patch_options(&original, &[(p::RATE_OFFSET, code)])?;
                if let Err(e) = self
                    .write(expected.to_vec())
                    .and_then(|_| self.verify_options(&expected))
                {
                    return self.restore_options(&original.try_into().unwrap(), e);
                }
                Ok(())
            }
            Action::Sleep(seconds) => {
                if seconds > 3600 {
                    return Err("休眠时间必须为 0–3600 秒".into());
                }
                let original = self.options()?;
                let edits = vec![
                    (p::SLEEP_24G_OFFSET, seconds as u8),
                    (p::SLEEP_24G_OFFSET + 1, (seconds >> 8) as u8),
                    (p::SLEEP_BT_OFFSET, seconds as u8),
                    (p::SLEEP_BT_OFFSET + 1, (seconds >> 8) as u8),
                ];
                // The core exposes one sleep value; mirror the upstream driver's policy by
                // setting both stored link timers, while preserving the rest of the option block.
                let expected = p::patch_options(&original, &edits)?;
                if let Err(e) = self
                    .write(expected.to_vec())
                    .and_then(|_| self.verify_options(&expected))
                {
                    return self.restore_options(&original.try_into().unwrap(), e);
                }
                Ok(())
            }
            Action::Light(_) | Action::Tune(_, _) => {
                Err("AJ179 PRO 此驱动没有确认该设置的写入协议".into())
            }
        }
    }
    fn read_controls(&mut self) -> Result<ControlState, String> {
        let matrix = self.keys()?;
        Ok(ControlState { binding_types:["mouse","disabled","dpi-cycle"].map(String::from).to_vec(), bindings:p::PHYSICAL_SLOTS.iter().map(|(control,slot,label)| {
            let raw=p::key_from_matrix(&matrix,*slot); let action=p::key_action(raw);
            let editable=!matches!(action,BindingAction::Unknown(_));
            Binding { control:*control,layer:0,label:(*label).into(),action,editable }
        }).collect(), macro_supported:false,notice:"仅编辑已识别的鼠标键、DPI 循环和禁用项；键盘/宏或未知原始键位保持只读。AJ179 PRO 的 Forward/Back 分别位于固件槽位 3/4。".into(),..ControlState::default() })
    }
    fn apply_control(&mut self, action: &ControlAction) -> Result<ControlState, String> {
        let (control, layer, desired) = match action {
            ControlAction::Bind {
                control,
                layer,
                action,
            } => (*control, *layer, action),
            _ => return Err("AJ179 PRO 没有已确认的宏或逐键灯协议".into()),
        };
        if layer != 0 {
            return Err("AJ179 PRO 没有第二改键层".into());
        }
        let (_, slot, _) = p::PHYSICAL_SLOTS
            .iter()
            .find(|(n, _, _)| *n == control)
            .ok_or("AJ179 PRO 按键编号无效")?;
        let wire = p::encode_key_action(desired)?;
        let original = self.keys()?;
        if matches!(
            p::key_action(p::key_from_matrix(&original, *slot)),
            BindingAction::Unknown(_)
        ) {
            return Err("当前按键包含未知原始配置，已拒绝覆盖".into());
        }
        if p::key_from_matrix(&original, *slot) == wire {
            return self.read_controls();
        }
        if control == 0
            && !(p::PHYSICAL_SLOTS.iter().any(|(n, s, _)| {
                *n != 0
                    && p::key_action(p::key_from_matrix(&original, *s)) == BindingAction::Mouse(1)
            }))
        {
            return Err("请至少保留一个左键功能".into());
        }
        let request = p::key_write(*slot, wire)?;
        let mut expected = original;
        expected[*slot * 4..*slot * 4 + 4].copy_from_slice(&wire);
        let update: Result<(), String> = (|| {
            self.write(request.to_vec())?;
            let actual = self.keys()?;
            if actual != expected {
                return Err("按键矩阵读回与预期不符，无法确认其他键位完整保留".into());
            }
            Ok(())
        })();
        if let Err(error) = update {
            let raw = p::key_from_matrix(&original, *slot);
            let rollback = p::key_write(*slot, raw)?;
            return match self
                .write(rollback.to_vec())
                .and_then(|_| self.keys())
                .and_then(|read| {
                    if read == original {
                        Ok(())
                    } else {
                        Err("原始按键矩阵读回不一致".into())
                    }
                }) {
                Ok(()) => Err(format!("{error}；已恢复并核对原按键")),
                Err(e) => Err(format!("{error}；恢复原按键失败：{e}")),
            };
        }
        self.read_controls()
    }
}

fn field<T: serde::de::DeserializeOwned>(request: &Value, name: &str) -> Result<T, String> {
    serde_json::from_value(
        request
            .get(name)
            .cloned()
            .ok_or_else(|| format!("缺少驱动参数：{name}"))?,
    )
    .map_err(|e| format!("驱动参数 {name} 无效：{e}"))
}
fn handler(key: &str, input: &str) -> String {
    let result = (|| {
        if key != p::DEVICE_KEY {
            return Err("驱动型号标识不匹配".to_string());
        }
        if input.len() > 1_048_576 {
            return Err("驱动请求过大".into());
        }
        let request: Value =
            serde_json::from_str(input).map_err(|e| format!("请求 JSON 无效：{e}"))?;
        let op = request
            .get("op")
            .and_then(Value::as_str)
            .ok_or("缺少驱动操作类型")?;
        if !matches!(
            op,
            "read-state" | "apply-setting" | "controls-read" | "controls-apply"
        ) {
            return Err("AJAZZ AJ179 PRO 不支持此操作".into());
        }
        let setting: Option<Action> = if op == "apply-setting" {
            Some(field(&request, "action")?)
        } else {
            None
        };
        let control: Option<ControlAction> = if op == "controls-apply" {
            Some(field(&request, "action")?)
        } else {
            None
        };
        let mut mouse = Mouse::open()?;
        let value = match op {
            "read-state" => serde_json::to_value(mouse.read_state()?).map_err(|e| e.to_string())?,
            "apply-setting" => {
                mouse.apply(setting.unwrap())?;
                Value::Null
            }
            "controls-read" => {
                serde_json::to_value(mouse.read_controls()?).map_err(|e| e.to_string())?
            }
            "controls-apply" => serde_json::to_value(mouse.apply_control(&control.unwrap())?)
                .map_err(|e| e.to_string())?,
            _ => unreachable!("operation was validated before opening HID"),
        };
        Ok(value)
    })();
    match result {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
peripheral_desk_driver_ffi::export_driver!("ajazz-aj179-pro", handler);
