fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_mchose_audio::handle(key, input)
}

peripheral_desk_driver_ffi::export_driver!("mchose-k20-pro", handler);
