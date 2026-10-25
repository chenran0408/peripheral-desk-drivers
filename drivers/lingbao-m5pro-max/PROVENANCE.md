# LINGBAO M5PRO Max

- Exact support: 2.4 GHz receiver `3151:402D`, usage page `FFFF`, usage `0002`, Feature report at least 65 bytes. Wired body `3151:4026` is explicitly excluded because it only echoes the query.
- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `LingbaoCodec.cs`, `LingbaoPlugin.cs`, `docs/hardware/lingbao.md` §1. The maintainer recorded real-device verification and corrected the command using official-driver runtime capture.
- Query is Feature report ID `00`, command `F7`, no checksum. Matching response requires `[0] == 0`, ACK `[1]` is `0` or `1`, return code `[2] == 0`, percent `[3]` is 2–100, and charging flag `[5]` is 0 or 1.
- This implementation is independently authored from Vigil Watt's public protocol facts; no upstream source code is copied. Peripheral Desk has not tested the device. Controls are not implemented.
