//! Ace 68 Air III, wired 41E4:2132.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x41e4,
        pid: 0x2132,
        kind: 202,
        travel_max: [
            340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "泰山磁轴 GT", 0),
            (2, "Kailh磁神轴", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-air-iii", handle);
