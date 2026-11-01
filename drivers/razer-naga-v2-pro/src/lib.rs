fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-naga-v2-pro",
        "Naga V2 Pro",
        0x00a8,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-naga-v2-pro", handler);
