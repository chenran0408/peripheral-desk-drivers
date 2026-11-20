//! Shared 8K receiver, paired body 0x3837:0x4036.
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_mchose_0x4d::request(key, input, 0x4036)
}

peripheral_desk_driver_ffi::export_driver!("mchose-r7-ultra", handler);
