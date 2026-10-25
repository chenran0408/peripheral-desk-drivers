# GameSir G7 Pro CE

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/gamesir.md` and `docs/hardware/protocols.md`; upstream contributor validated the dongle status flow against a physical G7 Pro CE.
- Exact collection guard: VID `3537`, usage page/usage `FFF0:0040`, 64-byte input/output. PID is intentionally not fixed because upstream records exact device revisions `10B9` and `0575` with the same report layout; the plugin does not match other VID/usage channels.
- Each read sends the documented `0F F2 00` heartbeat to enable a short status burst, then accepts report ID `12` only. Battery is byte 36 (integer percent); byte 35 equal to 1 means charging. No XInput battery API is used.
- This is active read-only communication: it sends the one documented heartbeat frame per read and does not write settings. The package must be shown as an active-query plugin and kept disabled until the user enables it.
- Implementation is independently written from the public protocol record; no vendor or Vigil Watt source code is copied. Peripheral Desk has not tested hardware.
