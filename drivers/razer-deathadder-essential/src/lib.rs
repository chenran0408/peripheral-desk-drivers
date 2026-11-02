use peripheral_desk_protocol_razer_wired_settings::{Profile, RateProtocol};
const PROFILE: Profile = Profile {
    key: "razer-deathadder-essential",
    model: "DeathAdder Essential",
    pid: 0x006e,
    dpi_max: 6400,
    dpi_transaction_id: 0xff,
    rate: RateProtocol::Standard {
        transaction_id: 0xff,
    },
};
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wired_settings::dispatch(key, input, PROFILE)
}
peripheral_desk_driver_ffi::export_driver!("razer-deathadder-essential", handler);
