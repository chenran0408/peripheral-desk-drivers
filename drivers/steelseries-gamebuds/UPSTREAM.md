# SteelSeries Arctis GameBuds

- [HeadsetControl GameBuds source](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/steelseries_arctis_gamebuds.hpp) and [Nova protocol](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/protocols/steelseries_protocol.hpp), GPLv3; referenced for `1038:230A`, FFC0:0001/interface 3, `00 B0`, active bud status byte 3/4 (`03`) and battery byte 5/6. Lower active bud level is shown. No source code copied.
- [HaloBattery SteelSeries provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py), MIT; corroborates active bud behavior. Case charge is not reported by wireless dongle.
- Code-adapter-only; Peripheral Desk has not tested hardware.
