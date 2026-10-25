# Pulsar X2 V2 receiver

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/pulsar.md`: upstream contributor verified the 2.4 GHz 1 kHz receiver `3554:F508` and a 20% battery/3570 mV response.
- Exact interface: `MI_01&Col05`, `FF02:0002`, 17-byte input and output, report ID `08`. This package does not match the wired body `F507` or 4 kHz receiver `F509`.
- Uses only the documented STATUS `03` and POWER `04` read queries. POWER byte `[6]` is integer 0–100, `[7]` is charging 0/1, and `[8..9]` is voltage mV big-endian. It does not use the optional event/push command.
- Protocol framing is independently authored from the Vigil Watt specification and Linux `pulsar` driver notes; no upstream source code is copied. Peripheral Desk has not tested this device.
- No DPI, report-rate, sleep, button, macro, or lighting settings are implemented by this battery adapter.
