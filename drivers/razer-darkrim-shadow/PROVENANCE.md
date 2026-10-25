# DARKRIM SHADOW receiver

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/darkrim.md`; contributor validated `3654:6901` at 90% against the official display.
- Exact interface: `3654:6901`, `FF02:0002`, MI_03, 65-byte IN/OUT (report ID 0). The same-usage 64-byte MI_04&Col02 does not respond and is excluded.
- The adapter sends only the documented read-only `GetBasicInfo` TLV 0x09 query. It validates HID report ID, magic, expected response command, source, TLV type/length and CRC-16/CCITT-FALSE before accepting battery/charging fields.
- Vigil Watt documents that the official `DARKRIM.exe` client can conflict with this single-consumer reply channel and cause headset/audio reconnection. This plugin checks for that process and yields without opening the HID interface when the client is running; if the process snapshot cannot be read, it also yields.
- No source code is copied. Peripheral Desk has not tested the device. This plugin does not expose EQ, key controls, firmware, lighting, or other writes.
