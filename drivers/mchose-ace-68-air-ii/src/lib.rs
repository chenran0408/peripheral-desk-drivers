//! Ace 68 Air II, wired 41E4:2120.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x41e4,
        pid: 0x2120,
        kind: 106,
        travel_max: [
            340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [109, 65535],
        switches: &[
            (0, "默认选项", 0),
            (1, "冰犀磁轴", 0),
            (2, "星芒磁轴", 0),
            (3, "KTEK MAX", 0),
            (4, "TTC 天王磁轴 (RGB 电竞版)", 0),
            (5, "TTC万磁王轴(兼容系列)", 0),
            (6, "火影磁轴", 0),
            (7, "Kailh磁神轴", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-air-ii", handle);
