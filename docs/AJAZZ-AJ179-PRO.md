# AJAZZ AJ179 PRO

Preview plugin for the AJ179 PRO using its GearHub-V5 feature protocol. It supports Windows USB cable mode (`3151:4026`) and the 2.4 GHz receiver (`3151:402D`). The shared product IDs do not uniquely identify the model, so the driver reads the device ID and proceeds only when it is `1851` (AJ179 PRO, PAW3395).

The plugin exposes battery percentage when the receiver reports an online mouse, current DPI and DPI stages, polling rate, sleep timer, and the core's existing `apply-setting`, `controls-read`, and `controls-apply` requests. It can change DPI, polling rate, sleep timer, active stage, and the six physical button assignments. Left/right/middle, back/forward, DPI cycle, and disabled mappings are editable. Unrecognized key entries, keyboard actions, and macros stay read-only and are not overwritten. AJ179 PRO maps Forward to firmware slot 3 and Back to slot 4.

The GearHub protocol is relay based on the receiver: select the mouse, wait for the receiver's send/read readiness, send a checksummed command, then request the reply. USB cable mode sends the same command directly. The device payload is an unnumbered 64-byte feature report. Windows HID calls include a leading report-ID byte set to zero, so the descriptor reports a 65-byte feature buffer. The driver reads the current setting blocks before editing them, preserves unused bytes and all DPI stages/colors, reads changes back, and attempts to restore and verify the original data after a failed write. Charging status is unknown. USB battery is not exposed. Bluetooth is not claimed in this Windows plugin.

The bundled mouse illustration is the project's existing neutral identification image, not an AJAZZ product photo.

## Evidence and source

The protocol implementation and sanitized firmware v3.03 capture in this package derive from the AGPL-3.0 `openmouse/mouse-protocol` repository at commit [`bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc). The capture is stored at `crates/protocol-ajazz-aj179-pro/tests/fixtures/ajazz-aj179-pro-v303.json`. The upstream implementation reports settings read/write and key read/write exercised on Windows 11 with firmware v3.03 over USB, receiver, and Bluetooth; this plugin implements only the two Windows Feature-report links described above. The capture is used only for protocol decoding fixtures; this plugin does not copy upstream application UI, browser code, HID paths, or user-specific identifiers.

Upstream evidence notes that the vendor's polling-rate field is a stored setting, not a measurement of Bluetooth input rate. Receiver rates through 8000 Hz appear in the device configuration, while USB is capped at 1000 Hz. This plugin accepts those stored receiver values and the USB limit; actual motion report rates are outside its readback contract. See the upstream [AJ179 PRO protocol notes](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/ajazz-aj179-pro.md).

## Limits

The plugin does not implement Bluetooth, macros, keyboard bindings, profile switching, stage-count editing, firmware updates, pairing, reset, calibration, or separate X/Y controls. It does not claim measured sensor resolution, lift-off behavior, LED appearance, standby timing, or physical effects of USB-written settings. Close the vendor configurator before use to avoid concurrent access to the same HID interface.
