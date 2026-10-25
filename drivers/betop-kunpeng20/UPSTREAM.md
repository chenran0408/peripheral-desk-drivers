# BETOP Kunpeng 20

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/betop.md` §1.3 and §1.8.
- Supported exact connections: 2.4 GHz dongle `20BC:5159` (passive status broadcast), wired body `20BC:5158` (same status query, passive listen then a documented read query when silent). Both require `00FF:0003`, 64-byte IN/OUT and product identity containing `KP20`.
- Parses report ID `02`, header `15`, byte 2 as integer battery percent, and only the low nibble of byte 4 as charging state. It does not infer any charge state for unknown low-nibble values.
- Upstream reports contributor hardware checks on the listed wireless/wired variants. The `20BC:5191` variant is excluded because its exact product identity/channel is not established by this plugin. Peripheral Desk has not tested hardware.
- Independently implemented from protocol notes; no upstream source code copied. No gamepad buttons, macros, lighting, or configuration writes are exposed.
