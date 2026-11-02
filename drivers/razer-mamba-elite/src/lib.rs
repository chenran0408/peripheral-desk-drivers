use peripheral_desk_protocol_razer_wired_settings::{Profile, RateProtocol};
const PROFILE: Profile = Profile {
    key: "razer-mamba-elite",
    model: "Mamba Elite",
    pid: 0x006c,
    dpi_max: 16000,
    dpi_transaction_id: 0x1f,
    rate: RateProtocol::Standard {
        transaction_id: 0xff,
    },
};
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wired_settings::dispatch(key, input, PROFILE)
}
peripheral_desk_driver_ffi::export_driver!("razer-mamba-elite", handler);
