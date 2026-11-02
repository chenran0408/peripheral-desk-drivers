# Scyrox V6

- Protocol facts: [VoltWalt/scyrox-v6-kde-battery `PROTOCOL.md`](https://github.com/VoltWalt/scyrox-v6-kde-battery/blob/master/PROTOCOL.md) and [`src/scyroxdevice.cpp`](https://github.com/VoltWalt/scyrox-v6-kde-battery/blob/master/src/scyroxdevice.cpp), both MIT licensed. The documentation includes live report captures for command 03 (online) and 04 (battery).
- Identity: wired `3554:F5F6`, 8K receiver `3554:F5F7`; report ID 08, 17-byte HID reports. Battery response carries level at byte 6, charging at byte 7, voltage at bytes 8-9, and a full-frame sum of 0x55.
- This adapter is independently implemented from protocol facts; no upstream source code is copied. Peripheral Desk has not tested hardware.
