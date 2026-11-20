//! Mix 87 I, wired 41E4:2122.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x41e4,
        pid: 0x2122,
        kind: 108,
        travel_max: [
            340, 340, 340, 340, 300, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 65535],
        switches: &[
            (0, "默认选项", 0),
            (1, "TTC万磁王轴(兼容系列)", 0),
            (2, "闪电磁轴", 0),
            (3, "KTEK MAX", 0),
            (4, "阿波罗磁轴", 0),
            (5, "泰山磁轴 GT", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-mix-87-i", handle);
