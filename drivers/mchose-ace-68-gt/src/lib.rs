//! Ace 68 GT, wired 3837:3007.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x3007,
        kind: 213,
        travel_max: [
            340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "粉泰山磁轴 GT", 0),
            (2, "泰山磁轴 GT", 0),
            (3, "宝马磁轴", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-gt", handle);
