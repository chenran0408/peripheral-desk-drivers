# Logitech G522 LIGHTSPEED

- Upstream: HeadsetControl commit `7b760ddb29e4618f4229ab2936a4b009ab2e1fa0`, `lib/devices/logitech_g522_lightspeed.hpp` and `lib/devices/protocols/logitech_centurion_protocol.hpp`.
- Reference license: GPL-3.0; this adapter uses protocol facts and does not copy upstream code.
- Scope: 046D:0B18 receiver, vendor collection FFA0:0001. Root lookup for bridge 0003, bridged Root lookup for battery 0104, then bridged battery function 0. Battery percentage at first response byte, charging state at third byte (1/2 = charging).
- Verification: HeadsetControl includes G522 wire captures and protocol fixtures; Peripheral Desk has no physical G522 validation.
