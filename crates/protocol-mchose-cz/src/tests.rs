use super::*;
use std::collections::HashMap;
#[derive(Default)]
struct Device {
    ram: HashMap<(u8, u16), u8>,
    fail: Option<(u8, u16)>,
    writes: usize,
}
impl Io for Device {
    fn packet(&mut self, cmd: u8, offset: u16, len: u8, write: Option<&[u8]>) -> Result<Vec<u8>> {
        if let Some(data) = write {
            let get = match cmd {
                6 => 5,
                9 => 8,
                161 => 160,
                163 => 162,
                165 => 164,
                167 => 166,
                _ => return Err("unexpected write".into()),
            };
            self.set(get, offset, data);
            self.writes += 1;
            if self.fail == Some((get, offset)) {
                self.fail = None;
                return Err("fixture dropped acknowledgement".into());
            }
            Ok(vec![])
        } else {
            Ok((offset..offset + u16::from(len))
                .map(|i| *self.ram.get(&(cmd, i)).unwrap_or(&0))
                .collect())
        }
    }
}
impl Device {
    fn set(&mut self, cmd: u8, offset: u16, data: &[u8]) {
        for (i, &v) in data.iter().enumerate() {
            self.ram.insert((cmd, offset + i as u16), v);
        }
    }
}
fn model() -> Model {
    Model {
        vid: 0x3837,
        pid: 0x3028,
        kind: 212,
        travel_max: [340; 16],
        precision_fw: [0, 0],
        switches: &[(0, "默认", 0)],
        keys: "[]",
    }
}
fn keys() -> Vec<Value> {
    vec![
        json!({"value":3,"label":"A","action":0x100004}),
        json!({"value":11,"label":"D","action":0x100007}),
    ]
}
fn ch(v: Value) -> Changes {
    v.as_object().unwrap().clone()
}
#[test]
fn rt_preserves_switch_priority_and_precision_bits() {
    let mut io = Device::default();
    io.set(160, 24, &[0xa2, 0x31, 20, 0x06, 20, 12, 45, 22]);
    let p = settings::page(
        &mut io,
        model(),
        0,
        3,
        "rt",
        &ch(json!({"act":1510,"mode":2})),
        &[0; 7],
    )
    .unwrap();
    assert_eq!(p["fields"][1]["value"], 1510);
    assert_eq!(
        wire::Region::new(160, 24, 8).read(&mut io).unwrap(),
        [0xa2, 0x32, 150, 6, 20, 12, 45, 22]
    );
}
#[test]
fn paired_socd_writes_both_keys_in_chosen_layer_and_undoes_lost_ack() {
    let mut io = Device::default();
    let key = advanced::matrix(1, 2, 3);
    let peer = advanced::matrix(1, 2, 11);
    for (k, b) in [(key, [16, 0, 4]), (peer, [16, 0, 7])] {
        io.set(7, k.addr, &b);
        io.set(8, k.addr, &b);
    }
    let sel = keys();
    let data = ch(json!({"enabled":1,"priority":1,"partner":11}));
    let before = io.ram.clone();
    io.fail = Some((8, key.addr));
    let result = advanced::page(&mut io, 1, 2, 3, "socd", &data, &sel);
    assert!(result.unwrap_err().contains("已恢复原值"));
    assert!(io
        .ram
        .iter()
        .all(|(k, v)| *v == *before.get(k).unwrap_or(&0)));
    advanced::page(&mut io, 1, 2, 3, "socd", &data, &sel).unwrap();
    assert_eq!(key.read(&mut io).unwrap(), [148, 0, 11]);
    assert_eq!(peer.read(&mut io).unwrap(), [148, 1, 3]);
    assert_eq!(
        wire::Region::new(164, 256, 12).read(&mut io).unwrap(),
        [16, 0, 4, 16, 0, 7, 16, 0, 7, 16, 0, 4]
    );
    assert_eq!(
        wire::Region::new(160, 1024 + 24, 2).read(&mut io).unwrap(),
        [0, 16]
    );
}
#[test]
fn dks_binds_single_action_and_unbinds_to_factory_key() {
    let mut io = Device::default();
    io.set(7, 9, &[16, 0, 4]);
    io.set(8, 9, &[16, 0, 4]);
    let sel = keys();
    let p = advanced::page(
        &mut io,
        0,
        0,
        3,
        "dks",
        &ch(json!({"enabled":1,"key0":0x100007,"phase0_0":1})),
        &sel,
    )
    .unwrap();
    assert_eq!(p["fields"][0]["value"], 1);
    assert_eq!(
        wire::Region::new(162, 0, 9).read(&mut io).unwrap(),
        [10, 30, 30, 10, 16, 0, 7, 1, 0]
    );
    advanced::page(&mut io, 0, 0, 3, "dks", &ch(json!({"enabled":0})), &sel).unwrap();
    assert_eq!(advanced::matrix(0, 0, 3).read(&mut io).unwrap(), [16, 0, 4]);
}
#[test]
fn template_preserves_opaque_data_and_round_trips_same_model() {
    let mut io = Device::default();
    let sel = keys();
    io.set(5, 4, &[4]);
    io.set(5, 35, &[2]);
    io.set(160, 24, &[0xa0, 0x31, 11, 0x80, 45, 10, 0, 8]);
    let snapshot = templates::handle(
        &mut io,
        "model",
        0,
        "mhub-magnetic-template-read",
        &json!({}),
        &sel,
        model(),
    )
    .unwrap();
    let mut doc = snapshot.clone();
    doc["travel"][26] = json!(67);
    doc["travel"][27] = json!(0);
    doc["performance"][0] = json!(199);
    templates::handle(
        &mut io,
        "model",
        0,
        "mhub-magnetic-template-apply",
        &json!({"document":doc}),
        &sel,
        model(),
    )
    .unwrap();
    assert_eq!(
        wire::Region::new(160, 26, 2).read(&mut io).unwrap(),
        [67, 0x80]
    );
    assert_eq!(wire::Region::new(5, 0, 1).read(&mut io).unwrap(), [0]);
    let copy = templates::handle(
        &mut io,
        "model",
        0,
        "mhub-magnetic-template-read",
        &json!({}),
        &sel,
        model(),
    )
    .unwrap();
    let r = templates::handle(
        &mut io,
        "model",
        0,
        "mhub-magnetic-template-apply",
        &json!({"document":copy}),
        &sel,
        model(),
    )
    .unwrap();
    assert_eq!(r["changed"], 0);
}

#[test]
fn vendor_profile_import_applies_only_listed_travel_and_performance_fields() {
    let mut io = Device::default();
    io.set(160, 24, &[0xa0, 0x21, 4, 0xe0, 90, 10, 85, 8]);
    io.set(5, 4, &[0xf4]);
    let doc = json!({"dataScope":"KeyboardProfile","version":3,"vendorId":0x3837,"productId":0x3028,"data":{"data":{"triggerTravel":{"travelKeys":[null,null,null,{"keyActuation":101,"rtPress":18}]},"performance":{"reporteRate":2}}}});
    let out = templates::handle(
        &mut io,
        "fixture",
        0,
        "mhub-magnetic-template-apply",
        &json!({"document":doc}),
        &keys(),
        model(),
    )
    .unwrap();
    assert_eq!(out["changed"], 2);
    assert_eq!(
        wire::Region::new(160, 24, 8).read(&mut io).unwrap(),
        [0xa0, 0x21, 100, 0xe0, 17, 10, 85, 8]
    );
    assert_eq!(wire::Region::new(5, 4, 1).read(&mut io).unwrap(), [0xf2]);
}
#[test]
fn combo_writes_to_selected_layer_and_restores_default() {
    let mut io = Device::default();
    let region = advanced::matrix(2, 3, 3);
    io.set(7, region.addr, &[16, 0, 4]);
    io.set(8, region.addr, &[16, 0, 4]);
    let sel = keys();
    advanced::page(
        &mut io,
        2,
        3,
        3,
        "cb",
        &ch(json!({"enabled":1,"modifier":2,"key":7})),
        &sel,
    )
    .unwrap();
    assert_eq!(region.read(&mut io).unwrap(), [16, 2, 7]);
    advanced::page(&mut io, 2, 3, 3, "cb", &ch(json!({"enabled":0})), &sel).unwrap();
    assert_eq!(region.read(&mut io).unwrap(), [16, 0, 4]);
}
