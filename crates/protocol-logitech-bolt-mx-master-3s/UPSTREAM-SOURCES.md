# Protocol sources and licensing

This crate is an independent Rust implementation of the public HID++ request/reply facts documented by Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), especially `docs/hardware/logitech.md` and the Bolt + MX Master 3S hardware validation dated 2026-10-08.

No Vigil Watt source code is copied. Vigil Watt is MIT-licensed; its source is retained here as protocol provenance, not as code attribution. HID++ protocol information also follows Logitech's public HID++ documentation and Solaar's public protocol descriptions; no GPL code from Solaar is included.

Supported identity is checked at runtime: receiver `046D:C548`, 20-byte long HID++ interface `FF00:0002`, and paired device name exactly `MX Master 3S`. Other Bolt receivers and other paired device names are ignored.
