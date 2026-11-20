//! Shared 8K receiver, paired body 0x3837:0x4037.
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_mchose_0x4d::request(key, input, 0x4037)
}

peripheral_desk_driver_ffi::export_driver!("mchose-k5-pro", handler);
