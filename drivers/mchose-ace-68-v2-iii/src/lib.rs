//! Ace 68 V2 III, wired 3837:3024.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x3024,
        kind: 210,
        travel_max: [
            340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "星环磁轴", 0),
            (2, "龙神磁轴", 0),
            (3, "青玉磁轴", 0),
            (5, "宝马磁轴", 0),
            (6, "闪电磁轴", 112),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-v2-iii", handle);
