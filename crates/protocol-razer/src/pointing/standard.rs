//! Keep the standard V3 Pro's quick slider, stage selector and per-axis editor
//! on the same board state. Its live DPI register is separate from its stages.
use super::{change, query_with_tid, write_with_tid, Device, Model, Stages, Value, TRANSACTION_ID};
use peripheral_desk::model::Action;

pub(super) enum Edit<'a> {
    Quick(u16),
    Select(u8),
    Pointer(&'a Value),
}

impl<'a> Edit<'a> {
    pub(super) fn legacy(op: &Value) -> Option<Self> {
        if op["op"] != "apply-setting" {
            return None;
        }
        match serde_json::from_value::<Action>(op["action"].clone()).ok()? {
            Action::Dpi(v) => Some(Self::Quick(v)),
            Action::Stage(n) => Some(Self::Select(n)),
            _ => None,
        }
    }
}

fn quick_xy(x: u16, y: u16, value: u16) -> Result<(u16, u16), String> {
    if !(100..=30000).contains(&value) || !value.is_multiple_of(50) {
        return Err("DPI 范围为 100 至 30000，步长 50".into());
    }
    Ok((value, if x == y { value } else { y }))
}
fn set_xy(raw: &mut [u8], x: u16, y: u16) {
    raw[1..3].copy_from_slice(&x.to_be_bytes());
    raw[3..5].copy_from_slice(&y.to_be_bytes());
}
fn live(device: &Device) -> Result<Vec<u8>, String> {
    let mut result = query_with_tid(device, TRANSACTION_ID, (4, 0x85, 7), &[1])?;
    result[0] = 1;
    Ok(result)
}
fn write_live(device: &Device, value: &[u8]) -> Result<(), String> {
    write_with_tid(device, TRANSACTION_ID, (4, 5, 7), value)
}
impl Stages {
    fn active_record(&self) -> usize {
        2 + 7 * usize::from(self.records[0] - 1)
    }
    fn active_xy(&self) -> (u16, u16) {
        let row = &self.records[self.active_record()..];
        (
            u16::from_be_bytes([row[1], row[2]]),
            u16::from_be_bytes([row[3], row[4]]),
        )
    }
    fn apply_standard(&mut self, edit: Edit<'_>) -> Result<bool, String> {
        match edit {
            Edit::Quick(v) => {
                let (x, y) = self.active_xy();
                let (x, y) = quick_xy(x, y, v)?;
                let at = self.active_record();
                set_xy(&mut self.records[at..], x, y);
                Ok(true)
            }
            Edit::Select(n) if n < self.records[1] => {
                self.records[0] = n + 1;
                Ok(true)
            }
            Edit::Select(_) => Err("档位不在设备的表中".into()),
            Edit::Pointer(op) => {
                self.edit(op, Model::V3Pro.max_dpi())?;
                Ok(op["stage"].as_u64() == Some(u64::from(self.records[0] - 1)))
            }
        }
    }
}

pub(super) fn apply(device: &Device, edit: Edit<'_>) -> Result<(), String> {
    let board = || Model::V3Pro.read_table(device);
    let before = match board() {
        Ok(table) => table,
        Err(e) => {
            // The older slider also works on the current DPI register when
            // firmware does not provide a readable stage table.
            let Edit::Quick(v) = edit else { return Err(e) };
            let original = live(device)?;
            let (x, y) = super::super::dpi_xy(&original)?;
            let (x, y) = quick_xy(x, y, v)?;
            let mut desired = original.clone();
            set_xy(&mut desired, x, y);
            return change(
                &original,
                &desired,
                |raw| write_live(device, raw),
                || live(device),
            );
        }
    };
    let mut target = before.clone();
    if !target.apply_standard(edit)? {
        return change(&before, &target, |t| t.write(device), board);
    }
    let original_live = live(device)?;
    let mut target_live = original_live.clone();
    let (x, y) = target.active_xy();
    set_xy(&mut target_live, x, y);
    let table_changed = before != target;
    let live_changed = original_live != target_live;
    change(
        &(before, original_live),
        &(target, target_live),
        |(table, current)| {
            if table_changed {
                table.write(device)?
            }
            if live_changed {
                write_live(device, current)?
            }
            Ok(())
        },
        || Ok((board()?, live(device)?)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quick_slider_and_selector_keep_inactive_and_split_axes() {
        let raw = [
            1, 1, 2, 1, 3, 32, 3, 32, 0x33, 0x12, 2, 6, 64, 7, 58, 0x95, 0x01,
        ];
        let mut report = [0xab; 38];
        report[..raw.len()].copy_from_slice(&raw);
        let mut s = Stages::parse(&report, Model::V3Pro).unwrap();
        s.apply_standard(Edit::Quick(950)).unwrap();
        assert_eq!(s.active_xy(), (950, 950));
        s.apply_standard(Edit::Select(1)).unwrap();
        assert_eq!(s.active_xy(), (1600, 1850));
        s.apply_standard(Edit::Quick(1700)).unwrap();
        assert_eq!(s.active_xy(), (1700, 1850));
        assert_eq!(s.axis(1), [950, 1700]);
        assert_eq!(s.axis(3), [950, 1850]);
        assert_eq!(s.records[14..], [0x95, 1]);
        assert_eq!(s.tail, [0xab; 21]);
    }
}
