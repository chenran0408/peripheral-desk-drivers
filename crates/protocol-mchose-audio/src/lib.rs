//! Model-specific M HUB audio settings. HID configuration is addressed to the
//! selected physical container, as are Windows render and capture endpoints.
mod endpoint;
mod models;
mod protocol;
mod speaker;
mod templates;
mod transport;
use endpoint::Endpoints;
use peripheral_desk::{hid, model::State};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};
use transport::{Channel, Physical};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Blk,
    McHeadset,
    Turbo,
    Oy,
    McSpeaker,
}
#[derive(Clone, Copy)]
struct Model {
    id: &'static str,
    product: &'static str,
    vid: u16,
    pid: u16,
    family: Family,
    wired: bool,
}
impl Model {
    fn speaker(self) -> bool {
        matches!(self.family, Family::Oy | Family::McSpeaker)
    }
    fn report(self) -> u8 {
        if self.speaker() {
            0xbc
        } else {
            0x55
        }
    }
    fn has_mode(self) -> bool {
        !matches!(self.id, "mchose-s9-pro" | "mchose-v9" | "mchose-g9")
    }
    fn allows_mode(self, physical: &Physical) -> Result<bool> {
        match self.id {
            "mchose-v9-pro" | "mchose-g9-pro" => {
                Ok(physical.blk_receiver_version()?
                    >= if self.id == "mchose-v9-pro" { 8 } else { 6 })
            }
            "mchose-g20" | "mchose-g20-pro" => Ok(physical.usb_rev().is_some_and(|n| n >= 160)),
            _ => Ok(self.has_mode()),
        }
    }
    fn sections(self) -> Vec<(&'static str, &'static str)> {
        let mut sections = vec![("playback", "播放")];
        if !self.speaker() {
            sections.push(("microphone", "麦克风"))
        }
        if self.has_mode() {
            sections.push(("mode", "音效模式"))
        }
        if self.family == Family::McHeadset {
            sections.extend([
                ("eq1", "均衡器 · 游戏一"),
                ("eq2", "均衡器 · 游戏二"),
                ("eq3", "均衡器 · 游戏三"),
            ])
        }
        if matches!(self.family, Family::McHeadset | Family::Turbo) {
            sections.push(("power", "设备设置"))
        }
        if matches!(self.id, "mchose-k20" | "mchose-k20-pro" | "mchose-k20-gt") {
            sections.push(("lighting", "灯光"))
        }
        if self.family == Family::McSpeaker {
            sections.push(("screen", "屏幕"))
        }
        sections
    }
}
#[derive(Default)]
struct AudioPage {
    fields: Vec<Value>,
    selector: Option<Value>,
    curve: Vec<Value>,
    note: String,
}
fn choice(value: i32, label: &str) -> Value {
    json!({"value":value,"label":label})
}
fn field(id: &str, label: &str, value: i32, min: i32, max: i32, unit: &str) -> Value {
    json!({"id":id,"label":label,"value":value,"min":min,"max":max,"unit":unit})
}
fn options(id: &str, label: &str, value: i32, items: &[(i32, &str)]) -> Value {
    json!({"id":id,"label":label,"value":value,"choices":items.iter().map(|(v,n)|choice(*v,n)).collect::<Vec<_>>()})
}
fn toggle(id: &str, label: &str, value: i32) -> Value {
    options(id, label, value, &[(0, "关"), (1, "开")])
}

enum Session {
    Endpoint(Endpoints),
    Board { ch: Channel, mode_enabled: bool },
}
impl Session {
    fn open(model: Model, physical: &Physical, section: &str) -> Result<Self> {
        if matches!(section, "playback" | "microphone") {
            Ok(Self::Endpoint(Endpoints::open(
                &physical.instance,
                section == "microphone",
            )?))
        } else {
            Ok(Self::Board {
                ch: physical.channel(model.report())?,
                mode_enabled: section != "mode" || model.allows_mode(physical)?,
            })
        }
    }
    fn read(&mut self, model: Model, section: &str, sel: Option<i32>) -> Result<AudioPage> {
        match self {
            Self::Endpoint(e) => e.page(sel),
            Self::Board { ch, mode_enabled } => match section {
                "mode" => protocol::mode_page(ch, model, *mode_enabled),
                "power" => protocol::power_page(ch, model),
                "lighting" => speaker::lights_page(ch, model),
                "screen" => speaker::screen_page(ch),
                "eq1" | "eq2" | "eq3" => protocol::eq_page(ch, eq_slot(section)?, sel),
                _ => Err("找不到当前设备的音频页面".into()),
            },
        }
    }
    fn write(
        &mut self,
        model: Model,
        section: &str,
        sel: Option<i32>,
        values: &BTreeMap<String, i32>,
    ) -> Result<()> {
        if let Self::Board { ch, .. } = self {
            if section == "lighting" {
                return speaker::lights_write(ch, model, values);
            }
            if section == "screen" {
                return speaker::screen_write(ch, values);
            }
            if section.starts_with("eq") {
                let mode = eq_slot(section)?;
                let i = sel
                    .and_then(|i| usize::try_from(i).ok())
                    .filter(|i| *i < 10)
                    .ok_or("缺少板载 EQ 频段")?;
                let mut gains = protocol::gains(ch, mode)?;
                for (id, val) in values {
                    if id != "gain" {
                        return Err("未知 EQ 选项".into());
                    }
                    gains[i] = *val;
                }
                return protocol::write_gains(ch, mode, &gains);
            }
        }
        for (id, val) in values {
            match self {
                Self::Endpoint(e) => e.selected(sel)?.ok_or("所选端点离线")?.write(id, *val)?,
                Self::Board { ch, .. } if section == "mode" => {
                    protocol::mode_write(ch, model, id, *val)?
                }
                Self::Board { ch, .. } if section == "power" => {
                    protocol::power_write(ch, model, id, *val)?
                }
                _ => return Err("所选音频设置不支持写入".into()),
            }
        }
        Ok(())
    }
}
fn eq_slot(section: &str) -> Result<u8> {
    match section {
        "eq1" => Ok(0),
        "eq2" => Ok(1),
        "eq3" => Ok(2),
        _ => Err("未知板载 EQ 槽".into()),
    }
}
fn as_i32(v: &Value) -> Option<i32> {
    v.as_i64().and_then(|n| i32::try_from(n).ok())
}
fn changes(
    page: &AudioPage,
    request: &Value,
) -> Result<(BTreeMap<String, i32>, BTreeMap<String, i32>)> {
    let changed = request.as_object().ok_or("缺少音频改动")?;
    let mut before = BTreeMap::new();
    let mut after = BTreeMap::new();
    for (id, val) in changed {
        let f = page
            .fields
            .iter()
            .find(|f| f["id"] == id.as_str())
            .ok_or("所选设备没有这个选项")?;
        let n = as_i32(val).ok_or("音频设置需要整数")?;
        if let Some(c) = f["choices"].as_array() {
            if !c.iter().any(|c| as_i32(&c["value"]) == Some(n)) {
                return Err("所选值超出设备选项".into());
            }
        } else if !(as_i32(&f["min"]).ok_or("缺少最小值")?
            ..=as_i32(&f["max"]).ok_or("缺少最大值")?)
            .contains(&n)
        {
            return Err("数值超出当前设置范围".into());
        }
        let old = as_i32(&f["value"]).ok_or("设备没有返回原值")?;
        if n != old {
            before.insert(id.clone(), old);
            after.insert(id.clone(), n);
        }
    }
    Ok((before, after))
}
fn confirmed(page: &AudioPage, values: &BTreeMap<String, i32>) -> Result<()> {
    if values.iter().all(|(id, v)| {
        page.fields
            .iter()
            .any(|f| f["id"] == id.as_str() && as_i32(&f["value"]) == Some(*v))
    }) {
        Ok(())
    } else {
        Err("设备读回值与写入值不一致".into())
    }
}
fn board_page(model: Model, physical: &Physical, request: &Value, apply: bool) -> Result<Value> {
    let sections = model.sections();
    let section = request["section"]
        .as_str()
        .filter(|x| !x.is_empty())
        .unwrap_or("playback");
    if !sections.iter().any(|(s, _)| *s == section) {
        return Err("设备未提供该音频设置".into());
    }
    let selector = if request["selector"].is_null() {
        None
    } else {
        Some(as_i32(&request["selector"]).ok_or("无效音频选项")?)
    };
    let read = || -> Result<AudioPage> {
        let mut io = Session::open(model, physical, section)?;
        let page = io.read(model, section, selector)?;
        if !apply {
            return Ok(page);
        }
        let select = page
            .selector
            .as_ref()
            .and_then(|s| as_i32(&s["value"]))
            .or(selector);
        if let Some(expected) = selector {
            if select != Some(expected) {
                return Err("当前板载槽或音频端点已切换".into());
            }
        }
        let (before, after) = changes(&page, &request["changes"])?;
        if after.is_empty() {
            return Ok(page);
        }
        if section == "lighting" && after.len() > 1 {
            if let (Some(&mode), Session::Board { ch, .. }) = (after.get("mode"), &mut io) {
                return speaker::switch_with_edits(ch, model, before["mode"], mode, after);
            }
        }
        let mut write = |values: &BTreeMap<String, i32>| {
            io.write(model, section, select, values)?;
            let result = io.read(model, section, select)?;
            confirmed(&result, values)?;
            Ok::<_, String>(result)
        };
        match write(&after) {
            Ok(v) => Ok(v),
            Err(error) => match write(&before) {
                Ok(_) => Err(format!("{error}；已恢复并读回修改前的值")),
                Err(e) => Err(format!("{error}；恢复未确认：{e}")),
            },
        }
    };
    let page = match read() {
        Ok(p) => p,
        Err(e) if !apply => AudioPage {
            note: e,
            ..Default::default()
        },
        Err(e) => return Err(e),
    };
    Ok(
        json!({"section":section,"sections":sections.iter().map(|(id,name)|json!({"id":id,"label":name})).collect::<Vec<_>>(),"selector":page.selector,"fields":page.fields,"curve":page.curve,"note":page.note}),
    )
}

fn process(key: &str, input: &str) -> Result<Value> {
    let model = *models::DEVICES
        .iter()
        .find(|m| m.id == key)
        .ok_or("未知音频型号")?;
    let req: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    let op = req["op"].as_str().ok_or("缺少音频操作")?;
    let target = hid::request_instance(&req)?;
    let physical = Physical::find(&model, target)?;
    if op == "read-state" {
        let mut state = State {
            key: key.into(),
            ..State::default()
        };
        if let Some(physical) = physical {
            state.present = true;
            state.wired = model.wired;
            state.identity = model.product.into();
            if !model.wired {
                let r = physical
                    .channel(model.report())
                    .and_then(|mut c| protocol::state(model, &mut c));
                match r {
                    Ok((p, c)) => {
                        state.percent = p;
                        state.charging = c;
                        state.fresh = p.is_some()
                    }
                    Err(e) => state.error = e,
                }
            } else {
                state.fresh = true
            }
            state.updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        return serde_json::to_value(state).map_err(|e| e.to_string());
    }
    let physical = physical.ok_or("所选型号或接收器目前未连接")?;
    match op {
        "mhub-audio-read" | "mhub-audio-apply" => {
            board_page(model, &physical, &req, op.ends_with("apply"))
        }
        "mhub-audio-template-read" | "mhub-audio-template-apply"
            if model.family == Family::McHeadset =>
        {
            templates::request(model, &physical, &req)
        }
        _ => Err("此设备未提供该项音频操作".into()),
    }
}
pub fn handle(key: &str, input: &str) -> String {
    match process(key, input) {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(e) => json!({"ok":false,"value":null,"error":e}),
    }
    .to_string()
}
