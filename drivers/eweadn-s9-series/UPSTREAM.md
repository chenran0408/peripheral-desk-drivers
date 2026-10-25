# EWEADN S9 Series

- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `EweadnMouseCodec.cs`, `EweadnPlugin.cs`, `docs/hardware/eweadn.md` §1.2.
- The supported HID endpoint is `372E:1026`, usage `FF00:0001`, Feature report ID `03`, descriptor Feature length 64. The shared PID covers S9 Pro/Max/Ultra and the upstream adapter uses their shared `beiyin_uuid` feature protocol; this package retains the upstream family scope rather than guessing a submodel.
- The plugin writes one Basics query to Feature report `03`, then reads the matching mailbox response. Battery is raw byte `[6]`, bit 7 means charging, lower 7 bits are the integer percentage. Zero and values above 100 are treated as unavailable.
- Vigil Watt's documentation records maintainer verification from a real S9-family receiver sample; Peripheral Desk has not tested this hardware.
- The adapter was independently authored from protocol documentation; no EWEADN or Vigil Watt implementation source code is copied. It exposes battery/charging only; DPI, remapping, macros, and lighting are not implemented.
