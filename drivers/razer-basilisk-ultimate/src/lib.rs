fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-basilisk-ultimate",
        "Basilisk Ultimate",
        0x0088,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-basilisk-ultimate", handler);
