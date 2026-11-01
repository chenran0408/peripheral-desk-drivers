fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-orochi-v2",
        "Orochi V2",
        0x0094,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-orochi-v2", handler);
