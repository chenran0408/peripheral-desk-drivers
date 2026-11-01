# SteelSeries Arctis 7+

- [HeadsetControl Arctis 7+ source](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/steelseries_arctis_7_plus.hpp) and [Nova protocol](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/protocols/steelseries_protocol.hpp), GPLv3; referenced for PIDs `1038:220E/2212/2216/2236`, FFC0:0001/interface 3, `00 B0`, reply status byte 1, battery byte 2 (`0..4`), charging byte 3. No source code copied.
- [HaloBattery SteelSeries provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py), MIT; corroborates the five battery levels (`0, 25, 50, 75, 100`). The task brief said nine levels, which conflicts with both current source files; this adapter follows the documented five levels.
- Code-adapter-only; Peripheral Desk has not tested hardware.
