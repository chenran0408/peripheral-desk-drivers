use peripheral_desk_protocol_razer_wired_settings::{Profile, RateProtocol};
const PROFILE: Profile = Profile {
    key: "razer-deathadder-v2",
    model: "DeathAdder V2",
    pid: 0x0084,
    dpi_max: 20000,
    dpi_transaction_id: 0x3f,
    rate: RateProtocol::Standard {
        transaction_id: 0x3f,
    },
};
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wired_settings::dispatch(key, input, PROFILE)
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-v2", handler);
