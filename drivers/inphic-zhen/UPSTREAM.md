# INPHIC ZHEN

- Exact receiver/interface: `1D57:FA65`, usage `FF00:0001`, 5-byte input reports, Report ID `03`. Do not match the separate `FF01:0002` output collection.
- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `InphicCodec.cs`, `InphicPlugin.cs`, `docs/hardware/inphic.md` §1. The source records local real-device validation with the official web driver.
- Passive report body is `76 41 <flag> <percent>` after the report ID. `01` means discharging, `03` means charging; other flags are unknown. Percent is the raw integer byte and must be 0–100.
- Independently authored from the documented protocol facts; no Vigil Watt or vendor source code copied. Peripheral Desk has not tested this hardware. No DPI, mapping, macro, or lighting settings are implemented.
