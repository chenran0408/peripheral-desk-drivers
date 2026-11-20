//! Zero75X, wired 41E4:211C.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x41e4,
        pid: 0x211c,
        kind: 123,
        travel_max: [
            340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 65535],
        switches: &[
            (0, "默认选项", 0),
            (1, "TTC天王磁轴(标准版)", 0),
            (2, "TTC天王磁轴(电竞版)", 0),
            (3, "TTC万磁王轴(兼容系列)", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-zero-75x", handle);
