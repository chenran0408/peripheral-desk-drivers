fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-mamba-wireless-receiver",
        "Mamba Wireless",
        0x0072,
        0x3f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-mamba-wireless-receiver", handler);
