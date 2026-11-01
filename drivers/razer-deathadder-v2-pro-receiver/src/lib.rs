fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-deathadder-v2-pro-receiver",
        "DeathAdder V2 Pro",
        0x007d,
        0x3f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v2-pro-receiver", handler);
