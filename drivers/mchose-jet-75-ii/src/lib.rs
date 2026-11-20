//! Jet 75 II, wired 41E4:211A.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x41e4,
        pid: 0x211a,
        kind: 122,
        travel_max: [
            340, 340, 340, 340, 340, 340, 340, 340, 350, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 65535],
        switches: &[
            (0, "默认选项", 0),
            (1, "TTC天王磁轴(标准版)", 0),
            (2, "TTC 天王磁轴 (RGB 电竞版)", 0),
            (3, "TTC万磁王轴(兼容系列)", 0),
            (4, "冰犀磁轴", 0),
            (5, "星猎磁轴", 0),
            (6, "星芒磁轴", 0),
            (7, "Kailh磁神轴", 0),
            (8, "蓝冰磁轴", 0),
            (9, "快银磁轴", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-jet-75-ii", handle);
