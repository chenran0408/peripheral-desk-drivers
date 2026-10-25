# MCHOSE G87 V2

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `MchoseV1Codec.cs`, `MchosePlugin.cs`, `docs/hardware/mchose.md` §1 and §1.6.
- Exact device scope: contributor-verified G87 V2 receiver `3837:201B`, keyboard V1 session channel `MI_01&Col01`, page `FF02`, 20-byte input/output, report ID `13`.
- Uses `13 05 01` to open the keyboard V1 session followed by `13 4A 01` to read battery. The contributor reported a 69% response. The session ACK model password is all zeros in this receiver; this plugin requires that observed signature and does not use the QHW `3837:3033` channel.
- Percent is read from response byte `[5]`; only valid checksummed `4A/01` replies with a value from 2–100 are accepted. Charging maps only documented flag bit `0x10` to charging and `0x01` to discharging; other values remain unknown.
- Upstream contributor hardware validation applies to the exact G87 V2 receiver and V1 session described above. Peripheral Desk has not tested this hardware.
- This adapter is independently authored from published protocol notes; no Vigil Watt source code is copied. Upstream provenance and license are recorded in the repository; no MIT source file is bundled.
- Controls are not exposed: the cited evidence verifies battery and session only, not end-to-end remapping, macro, or lighting writes for this receiver.
