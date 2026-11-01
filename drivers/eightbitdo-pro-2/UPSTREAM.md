# 8BitDo Pro 2 / SN30 Pro / SF30 Pro

- [HaloBattery 8BitDo provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/eightbitdo.py), MIT; referenced for `2DC8:6000/6100/6001/6101/6003/6006`, Bluetooth D-input input report `01`, battery byte 14 bits 0..6 and charging bit 7. The adapter never sends the feature report that switches the controller into enhanced reporting.
- [SDL 8BitDo HIDAPI driver](https://github.com/libsdl-org/SDL/blob/main/src/joystick/hidapi/SDL_hidapi_8bitdo.c), Zlib; corroborates enhanced report behavior. Ordinary reports with zero padding are not read as 0% battery.
- Code-adapter-only; HaloBattery tested a Pro 2 mode transition but not this adapter's battery read on these PIDs. The separate Pro 3 is excluded.
