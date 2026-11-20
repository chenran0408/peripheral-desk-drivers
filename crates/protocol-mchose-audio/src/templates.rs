use super::{
    as_i32, protocol,
    transport::{Channel, Physical},
    Model, Result,
};
use serde_json::{json, Value};

#[derive(Clone, PartialEq)]
struct EqSlot {
    gains: [i32; 10],
    name: String,
}
#[derive(Clone)]
struct Snapshot {
    active: u8,
    slots: Vec<EqSlot>,
}
fn capture(ch: &mut Channel) -> Result<Snapshot> {
    let active = protocol::active_eq(ch)?;
    let slots = (0..3)
        .map(|n| {
            Ok(EqSlot {
                gains: protocol::gains(ch, n)?,
                name: protocol::read_name(ch, n)?,
            })
        })
        .collect::<Result<_>>()?;
    Ok(Snapshot { active, slots })
}
impl Snapshot {
    fn json(&self, model: Model) -> Value {
        json!({"schema":1,"model":model.id,"active_slot":self.active,"slots":self.slots.iter().enumerate().map(|(i,v)|json!({"slot":i,"gain_tenths_db":v.gains,"name":v.name})).collect::<Vec<_>>()})
    }
    fn import(&self, model: Model, doc: &Value) -> Result<Self> {
        if doc["schema"] != 1 || doc["model"] != model.id {
            return Err("请选择同型号的板载 EQ 模板".into());
        }
        let mut result = self.clone();
        if let Some(v) = doc.get("active_slot") {
            result.active = v
                .as_u64()
                .filter(|n| *n < 3)
                .ok_or("模板中当前板载槽无效")? as u8;
        }
        let mut seen = [false; 3];
        if let Some(v) = doc.get("slots") {
            for item in v.as_array().ok_or("模板板载槽列表无效")? {
                let n = item["slot"]
                    .as_u64()
                    .filter(|n| *n < 3)
                    .ok_or("模板板载槽无效")? as usize;
                if std::mem::replace(&mut seen[n], true) {
                    return Err("模板含重复板载槽".into());
                }
                if let Some(value) = item.get("name") {
                    let name = value
                        .as_str()
                        .filter(|s| s.len() <= 54 && !s.contains('\0'))
                        .ok_or("板载名称最多 54 个 UTF-8 字节")?;
                    result.slots[n].name = name.into()
                }
                if let Some(value) = item.get("gain_tenths_db") {
                    let vals = value
                        .as_array()
                        .filter(|a| a.len() == 10)
                        .ok_or("模板需要十个 EQ 增益")?;
                    for (dst, src) in result.slots[n].gains.iter_mut().zip(vals) {
                        *dst = as_i32(src)
                            .filter(|n| (-120..=120).contains(n))
                            .ok_or("EQ 增益范围为 -12 至 +12 dB")?;
                    }
                }
            }
        }
        Ok(result)
    }
}
#[derive(Clone, Copy)]
enum Edit {
    Gains(usize),
    Name(usize),
    Active,
}
impl Edit {
    fn set(self, model: Model, ch: &mut Channel, s: &Snapshot) -> Result<()> {
        let ok = match self {
            Self::Gains(n) => {
                let val = &s.slots[n].gains;
                protocol::write_gains(ch, n as u8, val)?;
                protocol::gains(ch, n as u8)? == *val
            }
            Self::Name(n) => {
                let val = &s.slots[n].name;
                protocol::write_name(ch, n as u8, val)?;
                protocol::read_name(ch, n as u8)? == *val
            }
            Self::Active => {
                protocol::mode_write(ch, model, "mode", s.active.into())?;
                protocol::active_eq(ch)? == s.active
            }
        };
        if ok {
            Ok(())
        } else {
            Err("板载 EQ 读回值与写入值不同".into())
        }
    }
}
pub(super) fn request(model: Model, physical: &Physical, req: &Value) -> Result<Value> {
    let mut ch = physical.channel(model.report())?;
    let baseline = capture(&mut ch)?;
    if req["op"] == "mhub-audio-template-read" {
        return Ok(baseline.json(model));
    }
    let wanted = baseline.import(model, &req["document"])?;
    let mut writes = Vec::new();
    for (i, (a, b)) in baseline.slots.iter().zip(&wanted.slots).enumerate() {
        if a.gains != b.gains {
            writes.push(Edit::Gains(i))
        }
        if a.name != b.name {
            writes.push(Edit::Name(i))
        }
    }
    if baseline.active != wanted.active {
        writes.push(Edit::Active)
    }
    for (i, op) in writes.iter().enumerate() {
        if let Err(e) = op.set(model, &mut ch, &wanted) {
            let mut failures = Vec::new();
            for edit in writes[..=i].iter().rev() {
                if let Err(error) = edit.set(model, &mut ch, &baseline) {
                    failures.push(error)
                }
            }
            return if failures.is_empty() {
                Err(format!("{e}；涉及的原始板载设置均已恢复并读回"))
            } else {
                Err(format!(
                    "{e}；部分板载设置恢复失败：{}",
                    failures.join("、")
                ))
            };
        }
    }
    Ok(json!({"changed":writes.len(),"skipped":[]}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_template_preserves_other_slots_and_rejects_invalid_gain() {
        let model = super::super::models::DEVICES
            .iter()
            .find(|m| m.id == "mchose-r9")
            .copied()
            .unwrap();
        let baseline = Snapshot {
            active: 0,
            slots: vec![
                EqSlot {
                    gains: [-15; 10],
                    name: "我的配置".into()
                };
                3
            ],
        };
        let input = json!({"schema":1,"model":model.id,"slots":[{"slot":1,"name":"配置 B"}]});
        let item = baseline.import(model, &input).unwrap();
        assert_eq!(item.slots[1].gains, [-15; 10]);
        assert_eq!(item.slots[2].name, "我的配置");
        let mut bad = baseline.json(model);
        bad["slots"][1]["gain_tenths_db"][0] = json!(121);
        assert!(baseline.import(model, &bad).is_err());
    }
}
