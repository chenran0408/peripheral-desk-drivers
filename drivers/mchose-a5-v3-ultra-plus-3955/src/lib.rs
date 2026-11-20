//! Shared 8K receiver, paired body 0x3837:0x4034.
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_mchose_0x4d::request(key, input, 0x4034)
}

peripheral_desk_driver_ffi::export_driver!("mchose-a5-v3-ultra-plus-3955", handler);
