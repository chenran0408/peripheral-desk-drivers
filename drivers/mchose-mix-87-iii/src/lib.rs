//! Mix 87 III, wired 3837:300D.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x300d,
        kind: 201,
        travel_max: [
            340, 340, 340, 300, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "闪电磁轴", 0),
            (2, "KTEK MAX", 0),
            (3, "阿波罗磁轴", 0),
            (4, "泰山磁轴 GT", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-mix-87-iii", handle);
