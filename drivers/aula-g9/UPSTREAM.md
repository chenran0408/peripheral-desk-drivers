# AULA G9 headset receiver

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/aula.md` §3 and `docs/hardware/protocols.md`.
- Exact scope: receiver `38A6:0030`, vendor HID page `FFAB`, 64-byte status stream. The headset pushes `AB 47 4D 86 ...` once per second; this plugin only listens and never sends a query.
- Battery is raw integer byte `[6]`; charging is true only for documented flag value `0x02` in byte `[7]`. `(battery=0, flag=0)` means no reading, not 0%.
- Vigil Watt describes this as contributor-reported validation, not a project test. Peripheral Desk has not tested the device.
- The parser is independently authored from protocol notes; no external code copied. No audio EQ, lighting, volume, button, or macro controls are exposed.
