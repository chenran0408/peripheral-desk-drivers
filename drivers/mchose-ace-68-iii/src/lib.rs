//! Ace 68 III, wired 3837:3003.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x3003,
        kind: 203,
        travel_max: [
            340, 350, 340, 340, 340, 340, 340, 340, 340, 340, 340, 350, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "蓝冰磁轴", 0),
            (2, "冰犀磁轴", 0),
            (3, "星猎磁轴", 0),
            (4, "星芒磁轴", 0),
            (5, "星环磁轴", 0),
            (6, "TTC天王磁轴(电竞版)", 0),
            (7, "TTC 天王磁轴 (RGB 电竞版)", 0),
            (8, "TTC龙神轴", 0),
            (9, "TTC万磁王轴(兼容系列)", 0),
            (10, "KTEK MAX", 0),
            (11, "闪电磁轴", 113),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-iii", handle);
