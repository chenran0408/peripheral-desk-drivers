# ATK N9 V2 Ultimate

## Supported scope

This preview plugin reads battery percentage and the known charge state from the **ATK N9 V2 Ultimate 2.4 GHz receiver only**:

- Receiver VID/PID: `373B:1262`
- HID collection: usage page `FF90`, 64-byte input and output reports
- Request: report ID `55`, command `65`, target `01` (headset), sent through the interrupt OUT pipe (`WriteFile` semantics)
- Response: report ID `55`, command `65`, battery percent at byte 2 and charge state at byte 3
- Charge values: `02` discharging, `03` charging, `04` full, `1A` unknown

The full charge state is displayed as 100%. An unknown charge state remains unknown. This plugin does not provide EQ, virtual 7.1, lighting, macros, or other headset controls. The `373B:1262` identity alone is not used for the ATK A9 mouse; its collection and report lengths are distinct.

## Validation provenance

[Voltide v2.1.2](https://github.com/sdparklab/Voltide/releases/tag/v2.1.2), commit [`73f16fd2be44570c0a861cef6b4804d743656893`](https://github.com/sdparklab/Voltide/tree/73f16fd2be44570c0a861cef6b4804d743656893), lists this exact model and receiver as contributor hardware verified. Its [provider implementation](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKN9V2UltimateProvider.cs) records the report format and transport behavior.

The Peripheral Desk implementation is an independent Rust port and has not been independently tested against the headset. The plugin exposes only battery reading and known charging state. It makes no compatibility claim for another ATK headset, receiver, USB or Bluetooth mode, or firmware variant.

Voltide is MIT licensed; see its [license text](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/LICENSE). This plugin references its published protocol evidence but does not copy Voltide source code.
