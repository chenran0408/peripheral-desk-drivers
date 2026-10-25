# Sony DualSense Bluetooth

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/dualsense.md` and `docs/hardware/protocols.md`.
- Exact scope is physical Bluetooth HID `054C:0CE6`, Bluetooth HID service UUID 1124, usage `0001:0005`, 78-byte input reports. USB wired controllers, virtual HID, Edge and Haute42 Adapter 4DS are excluded.
- This plugin listens for complete Bluetooth report `0x31` and verifies the documented CRC32. Battery is a coarse 10% level; displayed integer is the upstream midpoint estimate. It sends no output or feature report.
- Upstream contributor tested a discharging level; other charge/full levels are covered by protocol documentation but not by that physical sample. Peripheral Desk has not tested hardware.
- The parser is independently authored from public protocol facts. No Linux driver code is copied; Linux source is cited by Vigil Watt as the report-format reference.
