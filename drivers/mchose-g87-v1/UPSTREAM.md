# MCHOSE G87 V1

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `MchoseV1Codec.cs`, `MchosePlugin.cs`, `docs/hardware/mchose.md` §1.
- Exact tested scope: G87 V1 receiver `41E4:2001`, `MI_01&Col01`, HID usage page `FF02`, 20-byte input/output, report ID `13`.
- Opens the legacy V1 session with `13 05 01 ... checksum`, then queries battery with `13 4A 01 ... checksum`. It accepts only matching command/parameter replies with a valid checksum and a percent from 2–100; `0x0A` notifications, other commands, and the unrelated `MI_01&Col06` feature mailbox are not parsed as battery.
- Charging is reported only for documented values: flag bit `0x10` means charging and `0x01` means discharging; other flag values are left unknown.
- Vigil Watt records this exact G87 V1 receiver as locally verified and in use. That is upstream validation; Peripheral Desk has not tested this hardware.
- QHW `3837:3033` and newer G87 V2 are explicitly outside this plugin. Their reports and command channel differ from the 20-byte legacy V1 interface.
- This adapter is independently authored from the published protocol notes; no Vigil Watt code is copied. Source facts are from the Vigil Watt MIT-licensed repository; no MIT source file is bundled.
- Controls are not exposed: the referenced G87 section verifies session, battery, and notifications, but does not provide a verified end-to-end write/read-back table for remapping, macros, or lighting on `41E4:2001`.
