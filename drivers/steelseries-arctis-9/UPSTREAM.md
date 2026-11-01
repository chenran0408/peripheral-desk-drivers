# SteelSeries Arctis 9

- [HeadsetControl Arctis 9 source](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/steelseries_arctis_9.hpp) and [legacy protocol](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/protocols/steelseries_protocol.hpp), GPLv3; referenced for `1038:12C2/12C4`, interface 0, `00 20`, byte 3 range `64..9A`, and byte 4 charging. No source code copied. HeadsetControl notes the later `12C4` revision is reported, not hardware-verified.
- [HaloBattery SteelSeries provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py), MIT; documents reply `AA 01` and disconnected replies.
- Code-adapter-only; Peripheral Desk has not tested hardware.
