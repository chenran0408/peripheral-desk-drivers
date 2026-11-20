//! Ace 60, wired 41E4:2101.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x41e4,
        pid: 0x2101,
        kind: 101,
        travel_max: [
            340, 340, 340, 340, 340, 340, 350, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [65535, 65535],
        switches: &[
            (0, "默认选项", 0),
            (1, "冰玉磁轴", 0),
            (2, "KTEK磁轴", 0),
            (3, "TTC天王磁轴(SE版)", 0),
            (4, "TTC天王磁轴(标准版)", 0),
            (5, "TTC万磁王轴", 0),
            (6, "蓝冰磁轴", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-60", handle);
