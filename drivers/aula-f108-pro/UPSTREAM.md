# AULA F108 Pro

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `GanssCodec.cs` / `GanssPlugin.cs`, `docs/hardware/ganss.md` §7.
- Exact shared-silicon scope: AULA F108 Pro or GANSS GS3104T-Pro receiver `05AC:024F`, channel `FF60:0061`, input/output 33 bytes, report ID 0, `MI_03` path or the model's product-name marker. Vigil Watt explicitly keeps both brands in one adapter because their HID signature is identical and two plugins could contend on the same interface. The plugin shows the brand only when the product string proves it; otherwise the identity remains shared/unknown.
- Query frame `00 20 01 ... 21`; parsed integer battery percent comes from the documented reply byte. No charging flag is inferred.
- Vigil Watt documents an additional contributor real-device check for F108 Pro; Peripheral Desk has not tested it. This USB VID is reused by another keyboard family, so the F108/product-path identity conditions remain required.
- Adapter is independently authored from the published protocol facts. No upstream source is copied.
- DPI, key mapping, macros, and lighting are not exposed. They require F108-specific control support from another upstream source.
