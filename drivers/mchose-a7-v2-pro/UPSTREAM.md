# MCHOSE A7 V2 Pro

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `MchoseMouseCodec.cs`, `MchosePlugin.cs`, `docs/hardware/mchose.md` §1.6.
- Exact device scope: contributor-verified A7 V2 Pro 2.4 GHz receiver `3837:100A`, MCHOSE mouse V2 vendor interface, report ID `13`, reports at least 60 bytes. Other products sharing the VID or a similar appearance are not included.
- Reads the passive E2 status frame: payload bytes are inverted after the unchanged report-ID byte; decoded `[3]` is charging (`1` means charging) and `[4]` is integer battery percent. Values 2–100 are accepted; other frames are ignored.
- Follows Vigil Watt's verified flow: listen up to 3 seconds for the receiver's periodic passive report; if none arrives, send the documented `0x11/0x06` status-wake frame and listen for another 3 seconds. This frame requests a status report; the plugin does not expose or write settings.
- Vigil Watt records contributor real-device validation of the E2 frame and trigger on this exact A7 V2 Pro receiver. Peripheral Desk has not tested this hardware.
- The adapter is independently authored from published protocol notes; no Vigil Watt code is copied. No MIT source file is bundled.
- Controls are not exposed: Vigil Watt's evidence here establishes device recognition, battery, charging, and a status trigger only; it does not establish DPI, report-rate, remapping, macro, or lighting support for this model.
