# EWEADN X87 keyboard receiver

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/eweadn.md` §1.1 and `docs/hardware/protocols.md`.
- Exact scope is receiver `0C45:FEFE`, vendor interface `FF60:0061`, 33-byte IN/OUT, report ID 0, path `MI_03`. `MI_04` is a different-length interface and is excluded.
- The query follows the published 33-byte request/checksum format. It only reads the battery percentage and charging is not reported upstream.
- Upstream contributor real-device validation covers X87 receiver; Peripheral Desk has not tested this adapter.
- The Rust parser/transport is independently authored from protocol facts. No upstream code copied. This adapter does not expose keyboard mapping, macros, lighting, or other writes.
