# SteelSeries Arctis 7 (2019)

- [HeadsetControl Arctis 7 source](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/steelseries_arctis_7.hpp) and [legacy protocol](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/protocols/steelseries_protocol.hpp), GPLv3; referenced for device IDs, FF43:0202/interface 5, request `06 18`, and response byte 2. No source code copied.
- [HaloBattery SteelSeries provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py), MIT; also checks link with `06 14` and treats a zero battery level as offline. Our single battery request follows HeadsetControl; zero is not shown.
- `1038:12AD`; code-adapter-only. Neither device nor Peripheral Desk adapter has been verified on hardware here.
