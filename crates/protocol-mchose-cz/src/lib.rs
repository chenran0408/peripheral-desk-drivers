mod advanced;
mod settings;
mod templates;
mod wire;

use peripheral_desk::{hid, model::State};
use peripheral_hid_transport::WinTransport;
use serde_json::{json, Value};
use wire::{Io, Result};

#[derive(Clone, Copy)]
pub struct Model {
    pub vid: u16,
    pub pid: u16,
    pub kind: u16,
    pub travel_max: [u16; 16],
    pub precision_fw: [u16; 2],
    pub switches: &'static [(i32, &'static str, u16)],
    pub keys: &'static str,
}
impl Model {
    fn smart(self) -> bool {
        matches!(self.kind,134..=136|138|200..=203|210..=214)
    }
    fn sleep(self) -> bool {
        matches!(self.kind,134..=136|138|211..=214)
    }
}
fn response(res: Result<Value>) -> String {
    match res {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}
pub fn request(key: &str, input: &str, model: Model) -> String {
    response(dispatch(key, input, model))
}
fn dispatch(key: &str, input: &str, model: Model) -> Result<Value> {
    let req: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    let target = hid::request_instance(&req)?;
    let mut endpoints = hid::enumerate()?.into_iter().filter(|i| {
        i.vid == model.vid
            && i.pid == model.pid
            && i.usage_page == 1
            && i.usage == 0
            && i.output_len == 65
            && i.input_len == 65
            && target.is_none_or(|id| i.instance_id() == id)
    });
    let i = endpoints.next().ok_or("未找到此键盘的有线配置接口")?;
    if endpoints.next().is_some() {
        return Err("请在设备调节中选择具体键盘".into());
    }
    let op = req["op"].as_str().unwrap_or("");
    if op == "read-state" {
        let state = State {
            key: key.into(),
            instance_id: i.instance_id(),
            present: true,
            fresh: true,
            wired: true,
            identity: format!("{:04X}:{:04X}", i.vid, i.pid),
            connection_path: Some("USB 有线".into()),
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ..State::default()
        };
        return serde_json::to_value(state).map_err(|e| e.to_string());
    }
    if !matches!(
        op,
        "mhub-magnetic-read"
            | "mhub-magnetic-apply"
            | "mhub-magnetic-template-read"
            | "mhub-magnetic-template-apply"
    ) {
        return Err("键盘未提供此操作".into());
    }
    let mut io = WinTransport::open(&i.path)?;
    io.flush()?;
    let slot = wire::current(&mut io)?;
    if req["profile"].as_u64().is_some_and(|v| v != slot as u64) {
        return Err("当前板载配置已切换，请重新读取".into());
    }
    let keys: Vec<Value> = serde_json::from_str(model.keys).map_err(|e| e.to_string())?;
    if op.starts_with("mhub-magnetic-template-") {
        return templates::handle(&mut io, key, slot, op, &req, &keys, model);
    }
    let section = match req["section"].as_str().unwrap_or("") {
        "" => "rt",
        s => s,
    };
    let index = match req["selector"].as_u64() {
        Some(i) if keys.iter().any(|k| k["value"].as_u64() == Some(i)) => i as usize,
        None => keys[0]["value"].as_u64().unwrap() as usize,
        _ => return Err("所选按键不属于该键盘布局".into()),
    };
    let changes = if op == "mhub-magnetic-apply" {
        req["changes"]
            .as_object()
            .ok_or("缺少要写入的字段")?
            .clone()
    } else {
        Default::default()
    };
    let scope = req["scope"].as_u64().unwrap_or(0);
    if scope > 3 {
        return Err("未知键位层".into());
    }
    let info = wire::Region::new(3, 0, 7).read(&mut io)?;
    let version = firmware(&info);
    let lt = match model.kind {
        213 => version >= 112,
        112 => version >= 122,
        211 => version >= 126,
        134..=136 | 138 => true,
        _ => false,
    };
    if section == "lt" && !lt {
        return Err("该键盘或固件未提供 LT".into());
    }
    let mut page = match section {
        "rt" | "performance" => {
            settings::page(&mut io, model, slot, index, section, &changes, &info)?
        }
        "dks" | "socd" | "rs" | "mt" | "tgl" | "lt" | "cb" => advanced::page(
            &mut io,
            slot,
            scope as usize,
            index,
            section,
            &changes,
            &keys,
        )?,
        _ => return Err("未知设置页".into()),
    };
    page["sections"] = json!([{"id":"rt","label":"触发设置 / RT"},{"id":"performance","label":"性能"},{"id":"dks","label":"动态键程 DKS"},{"id":"socd","label":"SOCD"},{"id":"rs","label":"迅捷 RS"},{"id":"mt","label":"轻点与长按 MT"},{"id":"tgl","label":"切换开关 TGL"},{"id":"cb","label":"组合键 CB"}]);
    if lt {
        page["sections"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"lt","label":"单键反向急停 LT"}));
    }
    page["profile"] = json!(slot);
    page["section"] = json!(section);
    if section != "performance" {
        page["selector"] = json!({"label":"选择按键","value":index,"choices":keys});
    }
    Ok(page)
}
fn firmware(info: &[u8]) -> u16 {
    format!("{:x}", u16::from_le_bytes([info[0], info[1]]))
        .parse()
        .unwrap_or(0)
}
type Changes = serde_json::Map<String, Value>;
fn choice(id: &str, label: &str, value: i32, options: Value) -> Value {
    json!({"id":id,"label":label,"value":value,"choices":options})
}
fn options(pairs: &[(i32, &str)]) -> Value {
    Value::Array(
        pairs
            .iter()
            .map(|(v, n)| json!({"value":v,"label":n}))
            .collect(),
    )
}
fn bit(id: &str, label: &str, value: bool) -> Value {
    choice(
        id,
        label,
        i32::from(value),
        options(&[(0, "关闭"), (1, "开启")]),
    )
}
fn number(id: &str, label: &str, value: i32, min: i32, max: i32) -> Value {
    json!({"id":id,"label":label,"value":value,"min":min,"max":max,"step":1})
}
fn validated(fields: &[Value], changes: &Changes) -> Result<Vec<(String, i32)>> {
    changes
        .iter()
        .map(|(k, v)| {
            let f = fields
                .iter()
                .find(|f| f["id"] == *k)
                .ok_or_else(|| format!("该页面未提供 {k}"))?;
            let n = v
                .as_i64()
                .and_then(|v| i32::try_from(v).ok())
                .ok_or("请输入整数")?;
            let valid = if let Some(options) = f["choices"].as_array() {
                options
                    .iter()
                    .any(|v| v["value"].as_i64() == Some(n.into()))
            } else {
                let min = f["min"].as_i64().unwrap_or(0);
                let max = f["max"].as_i64().unwrap_or(0);
                let step = f["step"].as_i64().unwrap_or(1).max(1);
                (min..=max).contains(&i64::from(n)) && (i64::from(n) - min) % step == 0
            };
            if !valid || f["readonly"] == true {
                return Err(format!(
                    "{} 的输入值不在允许范围",
                    f["label"].as_str().unwrap_or(k)
                ));
            }
            Ok((k.clone(), n))
        })
        .collect()
}
fn put_bits(byte: &mut u8, mask: u8, value: u8) {
    *byte = (*byte & !mask) | (value & mask)
}

#[cfg(test)]
mod tests;
