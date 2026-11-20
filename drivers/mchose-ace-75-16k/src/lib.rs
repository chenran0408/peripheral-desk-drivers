//! Ace 75 16K, wired 3837:301D.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x301d,
        kind: 135,
        travel_max: [
            340, 340, 340, 350, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "泰山磁轴 GT", 0),
            (2, "狂暴磁轴", 0),
            (3, "火影磁轴", 0),
            (4, "宝马磁轴", 0),
            (5, "星环磁轴V2", 117),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-75-16k", handle);
