# CHERRY PIXIU75 keyboard

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/cherry.md` and `docs/hardware/protocols.md`.
- Exact USB scope: 2.4 GHz receiver `046A:01E4` or wired keyboard `046A:01E2`; `FF1C:0092`, `MI_01&Col04`, 64-byte IN/OUT block channel, report ID `04`.
- The plugin reads block `0x1A`, offset 0, length 4. It requires the reply to echo the report, token, block and length; payload byte 0 is the raw integer percent (0–100), and payload byte 1 bit 0 is the documented charging flag.
- Vigil Watt records contributor real-device testing and official utility comparison for wired/wireless. Peripheral Desk has not tested this package.
- Protocol transport is independently authored from the documented public frame facts; no source code is copied. The plugin reads battery only and does not write keymap, macro, lighting, or configuration data.
