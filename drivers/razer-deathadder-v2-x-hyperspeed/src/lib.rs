fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wireless::dispatch_openrazer(
        key,
        input,
        "razer-deathadder-v2-x-hyperspeed",
        "DeathAdder V2 X HyperSpeed",
        0x009c,
        0x1f,
    )
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v2-x-hyperspeed", handler);
