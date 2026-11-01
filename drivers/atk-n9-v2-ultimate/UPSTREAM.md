# Upstream basis

- Project: [sdparklab/Voltide](https://github.com/sdparklab/Voltide)
- Release: [v2.1.2](https://github.com/sdparklab/Voltide/releases/tag/v2.1.2)
- Revision: `73f16fd2be44570c0a861cef6b4804d743656893`
- License: MIT, Copyright (c) 2026 Joko; full upstream text is available in [Voltide LICENSE](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/LICENSE).
- Protocol implementation: [ATKN9V2UltimateProvider.cs](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKN9V2UltimateProvider.cs).
- Upstream reports the N9 V2 Ultimate 2.4 GHz receiver protocol as contributor hardware verified on 2026-11-01. The report scope is receiver `373B:1262`, vendor usage page `FF90`, 64-byte input/output reports, report ID `55`, command `65`; the battery and charge fields are response bytes 2 and 3.
- This Peripheral Desk plugin is a modified, independent Rust port of the published protocol behavior; it does not copy Voltide source code. Copyright (c) 2026 Joko. Voltide's exact MIT license text is included at `THIRD_PARTY_LICENSES/Voltide-LICENSE.txt`. Peripheral Desk has not independently tested this hardware.

The 64-byte request is written through the HID interrupt OUT channel (`WriteFile` semantics), not HID `SetOutputReport`. Charge states `02`, `03`, `04`, and `1A` mean discharging, charging, full, and unknown. Full is presented as 100% as Voltide does; unknown charge remains unknown. No other ATK headset, connection mode, audio/EQ/7.1 effect, lighting feature, or configuration control is covered.
