# AULA L9 Ultra

- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `AulaL9Codec.cs`, `AulaPlugin.cs`, `docs/hardware/aula.md` §4.
- Exact supported receiver: `38A6:28FD`, usage page `FFA8`; only this endpoint from the receiver's vendor collections is queried.
- Query is report `0x0B`, command `0x81`; only matching input report `0x0A 0x81` is parsed. Charging flag is `[17]`, integer percent is `[18]`.
- Vigil Watt labels this contributor-reported hardware implementation; Peripheral Desk has not tested the device.
- This adapter is independently authored from the published protocol facts; no upstream source code is copied. It exposes only battery and charge state; DPI, polling, remapping, macros, and lighting are not implemented.
