fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-pro-click",
        "Pro Click",
        0x0077,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-pro-click", handler);
