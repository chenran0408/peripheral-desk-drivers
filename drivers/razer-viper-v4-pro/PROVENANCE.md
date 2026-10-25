# Viper V4 Pro protocol sources

This device adapter uses the exact Viper V4 Pro receiver and wired product IDs
(`1532:00E6` and `1532:00E5`) and the 91-byte Windows Feature-report transport.

- Battery and charging protocol, response layout, `0..255` battery scaling,
  and hardware validation scope: Vigil Watt v1.3.0, commit
  `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/razer.md` and
  `src/VigilWatt.Core/Plugins/Razer/RazerCodec.cs`.
- DPI stage table, polling-rate and sleep commands: OpenMouse
  `mouse-protocol`, commit `23f5bed75d148cabeed19c74838deb31d3f60d02`,
  `src/razer/v4.ts` and `src/drivers/razer/viper-v4-pro-hid.ts`.

The V4 protocol routines here are an AGPL-3.0-or-later Rust adaptation of the
OpenMouse implementation. See `THIRD_PARTY_LICENSES/AGPL-3.0.txt`.

Only the Viper V4 Pro IDs above are matched. DPI, polling and sleep commands
follow the OpenMouse model-specific implementation; upstream Vigil Watt's
hardware report covers battery and charging on the receiver and battery on the
wired model. This plugin has no key remapping, macro, lighting, or audio
controls: neither referenced implementation establishes those features for
this model.
