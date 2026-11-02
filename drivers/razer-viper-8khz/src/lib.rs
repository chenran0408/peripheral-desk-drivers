use peripheral_desk_protocol_razer_wired_settings::{Profile, RateProtocol};
const PROFILE: Profile = Profile {
    key: "razer-viper-8khz",
    model: "Viper 8KHz",
    pid: 0x0091,
    dpi_max: 20000,
    dpi_transaction_id: 0xff,
    rate: RateProtocol::Viper8k,
};
fn handler(key: &str, input: &str) -> String {
    peripheral_desk_protocol_razer_wired_settings::dispatch(key, input, PROFILE)
}
peripheral_desk_driver_ffi::export_driver!("razer-viper-8khz", handler);
