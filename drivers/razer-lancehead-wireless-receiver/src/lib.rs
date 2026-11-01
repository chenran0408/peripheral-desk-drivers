fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-lancehead-wireless-receiver",
        "Lancehead Wireless",
        0x006f,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-lancehead-wireless-receiver", handler);
