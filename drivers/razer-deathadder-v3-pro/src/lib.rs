use peripheral_desk_protocol_razer::pointing::{request, Model};

fn handler(key: &str, input: &str) -> String {
    request(
        Model::V3Pro,
        key,
        input,
        peripheral_desk_driver_ffi::core_razer_request,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v3-pro", handler);
