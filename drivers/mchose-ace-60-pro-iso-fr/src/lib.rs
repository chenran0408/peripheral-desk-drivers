//! Ace 60 Pro ISO FR, wired 3837:3040.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x3040,
        kind: 150,
        travel_max: [
            340, 340, 340, 350, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [65535, 65535],
        switches: &[(0, "默认选项", 0), (3, "蓝冰磁轴", 0)],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-60-pro-iso-fr", handle);
