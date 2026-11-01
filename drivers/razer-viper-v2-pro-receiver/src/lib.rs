fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-viper-v2-pro-receiver",
        "Viper V2 Pro",
        0x00a6,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-viper-v2-pro-receiver", handler);
