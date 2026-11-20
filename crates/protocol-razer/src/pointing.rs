//! Per-axis onboard settings for the three DeathAdder receiver variants.
//! Device and command facts are pinned in docs/RAZER-OPENSYNAPSE.md.
mod standard;

use super::{query_with_tid, request_with_tid, write_with_tid, RESPONSE_WAIT, TRANSACTION_ID, VID};
use peripheral_desk::{
    hid::{self, Device},
    model::State,
};
use serde_json::{json, Value};

#[derive(Clone, Copy)]
pub enum Model {
    V3Pro,
    HyperPolling,
    HyperSpeed,
}

impl Model {
    fn key(self) -> &'static str {
        match self {
            Self::V3Pro => "razer-deathadder-v3-pro",
            Self::HyperPolling => "razer-deathadder-v3-pro-alt",
            Self::HyperSpeed => "razer-deathadder-v3-hyperspeed",
        }
    }
    fn pids(self) -> &'static [u16] {
        match self {
            Self::V3Pro => &[0x00b6, 0x00b7],
            Self::HyperPolling => &[0x00c3],
            Self::HyperSpeed => &[0x00c5],
        }
    }
    fn max_dpi(self) -> u16 {
        match self {
            Self::HyperSpeed => 26000,
            _ => 30000,
        }
    }
    fn rates(self) -> &'static [u16] {
        match self {
            Self::HyperPolling => &[125, 500, 1000, 2000, 4000, 8000],
            _ => &[125, 500, 1000],
        }
    }
    fn rate_base(self) -> u16 {
        match self {
            Self::HyperPolling => 8000,
            _ => 1000,
        }
    }
    fn open(self) -> Result<Device, String> {
        let list = hid::enumerate()?;
        // Follow the core device page: use the cable first when a standard
        // receiver remains plugged in. Never choose between two identical PIDs.
        for pid in self.pids() {
            let mut matches = list.iter().filter(|i| {
                i.vid == VID
                    && i.pid == *pid
                    && i.feature_len == 91
                    && i.path.to_ascii_lowercase().contains("mi_00")
            });
            if let Some(i) = matches.next() {
                if matches.next().is_some() {
                    return Err("连接了多个同型号设备，请只保留要编辑的一台".into());
                }
                return Device::open(&i.path);
            }
        }
        Err("未找到此型号的控制接口，请连接鼠标".into())
    }
    fn read_table(self, device: &Device) -> Result<Stages, String> {
        let (size, storage) = match self {
            Self::V3Pro => (38, 0),
            _ => (80, 1),
        };
        let request = request_with_tid(TRANSACTION_ID, 4, 0x86, size, &[storage])?;
        device.send(&request)?;
        std::thread::sleep(if matches!(self, Self::V3Pro) {
            std::time::Duration::from_millis(80)
        } else {
            RESPONSE_WAIT
        });
        let response = device.receive(0, 91)?;
        // Some firmware reports only the occupied bytes in the GET reply.
        let body = stage_reply(&request, &response)?;
        Stages::parse(body, self)
    }
    fn read_rate(self, device: &Device) -> Result<u16, String> {
        let high = matches!(self, Self::HyperPolling);
        let body = if high {
            query_with_tid(device, TRANSACTION_ID, (0, 0xc0, 2), &[1])?
        } else {
            query_with_tid(device, TRANSACTION_ID, (0, 0x85, 1), &[])?
        };
        let raw = body[usize::from(high)];
        self.rates()
            .iter()
            .copied()
            .find(|hz| self.rate_base() / hz == u16::from(raw))
            .ok_or_else(|| format!("此型号未声明回报率编码 0x{raw:02X}"))
    }
    fn write_rate(self, device: &Device, hz: u16) -> Result<(), String> {
        if !self.rates().contains(&hz) {
            return Err("不支持此回报率".into());
        }
        let code = (self.rate_base() / hz) as u8;
        if matches!(self, Self::HyperPolling) {
            write_with_tid(device, TRANSACTION_ID, (0, 0x40, 2), &[1, code])
        } else {
            write_with_tid(device, TRANSACTION_ID, (0, 5, 1), &[code])
        }
    }
}

fn stage_reply<'a>(request: &[u8; 91], reply: &'a [u8]) -> Result<&'a [u8], String> {
    if reply.len() != 91
        || reply[0] != 0
        || reply[1] != 2
        || reply[2..6] != request[2..6]
        || reply[7..9] != request[7..9]
        || reply[6] > 80
        || reply[89] != reply[3..89].iter().fold(0, |v, b| v ^ b)
    {
        return Err("DPI 档位回复未成功或与本次请求不匹配".into());
    }
    Ok(&reply[9..9 + usize::from(reply[6])])
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Stages {
    // Begins at the active-stage byte; includes slot identifiers, both axes,
    // Z/reserved bytes and only the records the firmware says it uses.
    records: Vec<u8>,
    // The standard V3 Pro uses a fixed 38-byte write; retain unused slot bytes.
    tail: Vec<u8>,
}
impl Stages {
    fn parse(body: &[u8], model: Model) -> Result<Self, String> {
        if body.len() < 3 || !(1..=5).contains(&body[2]) || !(1..=body[2]).contains(&body[1]) {
            return Err("设备的 DPI 档位表头无法识别".into());
        }
        let len = 3 + 7 * usize::from(body[2]);
        let result = Self {
            records: body.get(1..len).ok_or("DPI 档位回复不完整")?.to_vec(),
            tail: if matches!(model, Model::V3Pro) && body.len() == 38 {
                body[len..].to_vec()
            } else {
                Vec::new()
            },
        };
        for row in result.records[2..].as_chunks::<7>().0 {
            for bytes in [&row[1..3], &row[3..5]] {
                if !(100..=model.max_dpi()).contains(&u16::from_be_bytes([bytes[0], bytes[1]])) {
                    return Err("档位表含未知 DPI 值".into());
                }
            }
        }
        Ok(result)
    }
    fn axis(&self, offset: usize) -> Vec<u16> {
        self.records[2..]
            .as_chunks::<7>()
            .0
            .iter()
            .map(|r| u16::from_be_bytes([r[offset], r[offset + 1]]))
            .collect()
    }
    fn edit(&mut self, op: &Value, max: u16) -> Result<(), String> {
        let value = op["value"]
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .ok_or("请输入有效整数")?;
        match op["field"].as_str() {
            Some("dpi") => {
                let offset = match op["axis"].as_str() {
                    Some("x") => 1,
                    Some("y") => 3,
                    _ => return Err("请选择 X 或 Y 轴".into()),
                };
                let slot = op["stage"]
                    .as_u64()
                    .filter(|n| *n < u64::from(self.records[1]))
                    .ok_or("档位不在设备的表中")? as usize;
                if !(100..=max).contains(&value) || !value.is_multiple_of(50) {
                    return Err(format!("DPI 范围为 100 至 {max}，步长 50"));
                }
                let at = 2 + slot * 7 + offset;
                self.records[at..at + 2].copy_from_slice(&value.to_be_bytes());
            }
            Some("active_stage") if (1..=u16::from(self.records[1])).contains(&value) => {
                self.records[0] = value as u8
            }
            _ => return Err("此设备没有这个指针设置项".into()),
        }
        Ok(())
    }
    fn write(&self, device: &Device) -> Result<(), String> {
        let mut body = vec![1];
        body.extend_from_slice(&self.records);
        body.extend_from_slice(&self.tail);
        write_with_tid(device, TRANSACTION_ID, (4, 0x06, body.len() as u8), &body)
    }
}

fn change<T: PartialEq>(
    before: &T,
    after: &T,
    write: impl Fn(&T) -> Result<(), String>,
    read: impl Fn() -> Result<T, String>,
) -> Result<(), String> {
    if before == after {
        return Ok(());
    }
    let write_result = write(after);
    let error = match read() {
        Ok(v) if &v == after => return Ok(()),
        Ok(v) if &v == before => {
            return Err(format!(
                "{}；已确认原设置仍在",
                write_result.err().as_deref().unwrap_or("写入未生效")
            ));
        }
        result => write_result
            .err()
            .or_else(|| result.err())
            .unwrap_or_else(|| "设备读回与写入值不同".into()),
    };
    let restore = write(before);
    match read() {
        Ok(v) if &v == before => Err(format!("{error}；原设置已恢复并核对")),
        result => {
            let why = restore
                .err()
                .or_else(|| result.err())
                .unwrap_or_else(|| "读回与原值不同".into());
            Err(format!("{error}；未能确认恢复：{why}"))
        }
    }
}
fn sample(model: Model, device: &Device) -> Result<Value, String> {
    let mut state = State {
        key: model.key().into(),
        present: true,
        ..State::default()
    };
    let mut y = None;
    let mut controls = Vec::new();
    match model.read_table(device) {
        Ok(table) => {
            state.dpi_stages = table.axis(1);
            y = Some(table.axis(3));
            state.active_stage = Some(table.records[0] - 1);
            state.dpi = state
                .dpi_stages
                .get(usize::from(table.records[0] - 1))
                .copied();
            if !matches!(model, Model::V3Pro) {
                let count = table.records[1];
                controls.push(json!({"field":"active_stage","label":"当前 DPI 档位","value":table.records[0],"min":1,"max":count,"step":1,"choices":(1..=count).map(|n|json!({"value":n,"label":format!("档位 {n}")})).collect::<Vec<_>>()}));
            }
        }
        Err(e) => state.settings_error = e,
    };
    match model.read_rate(device) {
        Ok(hz) => state.rate = Some(hz),
        Err(e) => {
            if !state.settings_error.is_empty() {
                state.settings_error.push_str(" · ")
            }
            state.settings_error.push_str(&e)
        }
    }
    state.fresh = !state.dpi_stages.is_empty() || state.rate.is_some();
    if !state.fresh {
        return Err(state.settings_error);
    }
    let rate_available = state.rate.is_some();
    let mut out = serde_json::to_value(state).map_err(|e| e.to_string())?;
    out["pointer_profile"] = json!(0);
    out["dpi_min"] = json!(100);
    out["dpi_max"] = json!(model.max_dpi());
    out["dpi_step"] = json!(50);
    out["dpi_stages_y"] = json!(y);
    out["pointer_controls"] = json!(controls);
    if rate_available {
        out["supported_rates"] = json!(model.rates());
    }
    Ok(out)
}
fn run(model: Model, op: &Value) -> Result<Value, String> {
    let device = model.open()?;
    if op["op"] == "read-state" {
        return sample(model, &device);
    }
    if matches!(model, Model::V3Pro) {
        if let Some(edit) = standard::Edit::legacy(op) {
            return standard::apply(&device, edit).map(|()| Value::Null);
        }
    }
    if op["profile"].as_u64() != Some(0) {
        return Err("此鼠标仅提供一个板载配置".into());
    }
    if op["field"] == "rate" {
        let hz = op["value"]
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .filter(|hz| model.rates().contains(hz))
            .ok_or("不支持此回报率")?;
        let before = model.read_rate(&device)?;
        change(
            &before,
            &hz,
            |n| model.write_rate(&device, *n),
            || model.read_rate(&device),
        )?;
    } else if matches!(model, Model::V3Pro) {
        standard::apply(&device, standard::Edit::Pointer(op))?;
    } else {
        let before = model.read_table(&device)?;
        let mut after = before.clone();
        after.edit(op, model.max_dpi())?;
        change(
            &before,
            &after,
            |table| table.write(&device),
            || model.read_table(&device),
        )?;
    }
    Ok(Value::Null)
}

pub fn request(model: Model, key: &str, input: &str, other: fn(&str, &str) -> String) -> String {
    if key != model.key() {
        return json!({"ok":false,"value":null,"error":"驱动型号不匹配"}).to_string();
    }
    let Ok(op) = serde_json::from_str::<Value>(input) else {
        return other(key, input);
    };
    let legacy_pointer = matches!(model, Model::V3Pro) && standard::Edit::legacy(&op).is_some();
    if !legacy_pointer
        && op["op"] != "set-pointer"
        && !(op["op"] == "read-state" && op["read_pointer"] == true)
    {
        return other(key, input);
    }
    match run(model, &op) {
        Ok(value) => json!({"ok":true,"value":value,"error":""}),
        Err(error) => json!({"ok":false,"value":null,"error":error}),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editing_inactive_y_preserves_stage_x_and_unknown_bytes() {
        let body = [
            1, 1, 2, 1, 3, 32, 3, 32, 0x91, 0x14, 2, 6, 64, 6, 64, 0x85, 0x07,
        ];
        let mut table = Stages::parse(&body, Model::V3Pro).unwrap();
        table
            .edit(
                &json!({"field":"dpi","axis":"y","stage":1,"value":1850}),
                30000,
            )
            .unwrap();
        let mut expected = body[1..].to_vec();
        expected[12..14].copy_from_slice(&1850u16.to_be_bytes());
        assert_eq!(table.records, expected);
        assert_eq!(table.axis(1), [800, 1600]);
        assert_eq!(table.axis(3), [800, 1850]);
    }
    #[test]
    fn missed_write_ack_does_not_undo_a_verified_edit() {
        use std::cell::Cell;
        let state = Cell::new(800);
        let writes = Cell::new(0);
        let result = change(
            &800,
            &900,
            |value| {
                writes.set(writes.get() + 1);
                state.set(*value);
                Err("missing ack".into())
            },
            || Ok(state.get()),
        );
        assert!(result.is_ok());
        assert_eq!((state.get(), writes.get()), (900, 1));
    }
    #[test]
    fn rates_distinguish_hyperpolling_from_ordinary_receivers() {
        assert_eq!(Model::HyperPolling.rate_base() / 1000, 8);
        assert_eq!(Model::HyperSpeed.rate_base() / 1000, 1);
        assert!(!Model::V3Pro.rates().contains(&8000));
    }
}
