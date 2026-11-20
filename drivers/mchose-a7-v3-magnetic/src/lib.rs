//! Shared 8K receiver, paired body 0x3837:0x403f.
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_mchose_0x4d::request(key, input, 0x403f)
}

peripheral_desk_driver_ffi::export_driver!("mchose-a7-v3-magnetic", handler);
