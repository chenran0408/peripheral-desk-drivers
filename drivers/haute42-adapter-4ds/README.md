# Haute42 Adapter 4DS

Adapted from Vigil Watt v1.3.0 `HautePlugin.cs` and `HauteCodec.cs`, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, under MIT; license text is retained with the protocol crate and plugin package. The exact upstream scope is the USB adapter that identifies as `054C:0CE6`, usage `1:5`, has Feature reports, and returns a firmware string beginning `Haute42 `. Bluetooth DualSense is explicitly excluded.

The plugin performs only zero-write `GetFeature` reads for firmware identity, chip ID and battery. Vigil Watt contributor validation is documented upstream; Peripheral Desk has not tested hardware. It reports the raw integer percent and charge/discharge when defined. `0xFF` percent means no current reading; charge flag `2` means full but Peripheral Desk's current boolean field cannot express “full”, so the plugin leaves charge unknown in that case. No button mapping, macro or lighting writes are included.
