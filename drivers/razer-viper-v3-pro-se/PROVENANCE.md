# Razer Viper V3 Pro SE provenance

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/razer-viper-se.md` and `RazerCodec` battery profiles: contributor real-device evidence applies to the wireless receiver only, PID `1532:00DF`, MI_00, usage page 0x0001/usage 0x0002, 91-byte Feature channel. It does not cover wired `00DE`, ordinary Viper V3 Pro `00C1`, or HyperPolling receiver variants.
- OpenMouse `mouse-protocol`, commit `23f5bed75d148cabeed19c74838deb31d3f60d02`, `src/razer/devices.ts`: model-specific SE profile identifies 35,000 DPI maximum and 125/500/1000 Hz receiver polling. The plugin uses the model's DPI and polling commands only for exact PID `00DF`; no sleep, remapping, macros, or lighting are exposed because the model-specific source does not establish those settings.
- OpenMouse is AGPL-3.0-or-later. Adapted implementation is covered by that license; text is included at `THIRD_PARTY_LICENSES/AGPL-3.0.txt`.

Vigil Watt's battery report is upstream contributor hardware evidence. The DPI/rate commands are derived from OpenMouse's protocol implementation; Peripheral Desk has not hardware-tested this adapter. The stock receiver is capped at 1000 Hz. No controls are inferred for other IDs or connection modes.
