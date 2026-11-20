//! Ace 68 Air 2, wired 3837:300A.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x300a,
        kind: 214,
        travel_max: [
            340, 340, 340, 350, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "泰山磁轴 GT", 0),
            (2, "狂暴磁轴", 0),
            (3, "极地磁轴", 116),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-air-2", handle);
