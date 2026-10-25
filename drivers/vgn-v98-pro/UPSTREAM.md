# VGN V98 Pro

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `VgnV98ProCodec.cs` / `VgnPlugin.cs`, `docs/hardware/vgn.md` §3.
- Exact scope: `320F:5088`, usage page/usage `FF1C:0092`, output report at least 8 bytes; query report ID `04`, command `00 00 1A 06 00 00 00`; response percent `[8]`, charging flag `[9]`.
- Vigil Watt calls this contributor-reported verification; Peripheral Desk has not tested it.
- Adapter is independently authored from the documented protocol; no upstream source code copied.
- No DPI, remap, macro or lighting control is claimed. These are not implemented by Vigil Watt's V98 battery codec; a verified device-specific control project and model mapping remain to be integrated.
