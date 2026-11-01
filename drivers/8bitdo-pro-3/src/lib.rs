// placeholder: implemented by integration worker
fn handler(_key: &str, _input: &str) -> String {
    serde_json::json!({"ok":false,"value":null,"error":"driver not implemented"}).to_string()
}
peripheral_desk_driver_ffi::export_driver!("8bitdo-pro-3", handler);
