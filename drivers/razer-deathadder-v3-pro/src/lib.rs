use peripheral_desk_protocol_razer::pointing::{request_battery_only, Model};

fn handler(key: &str, input: &str) -> String {
    request_battery_only(Model::V3Pro, key, input)
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v3-pro", handler);
