//! Ace 68 Turbo 8K, wired 3837:3028.
use peripheral_desk_protocol_mchose_cz::{request, Model};

fn handle(key: &str, input: &str) -> String {
    let model = Model {
        vid: 0x3837,
        pid: 0x3028,
        kind: 212,
        travel_max: [
            340, 340, 340, 330, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340, 340,
        ],
        precision_fw: [0, 0],
        switches: &[
            (0, "默认选项", 0),
            (1, "泰山磁轴 GT", 0),
            (2, "北极星磁轴", 0),
            (3, "佳达隆磁玉Pro轴", 0),
        ],
        keys: include_str!("keys.json"),
    };
    request(key, input, model)
}

peripheral_desk_driver_ffi::export_driver!("mchose-ace-68-turbo-8k", handle);
