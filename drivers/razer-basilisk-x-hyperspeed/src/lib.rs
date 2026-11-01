fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-basilisk-x-hyperspeed",
        "Basilisk X HyperSpeed",
        0x0083,
        0xff,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-basilisk-x-hyperspeed", handler);
