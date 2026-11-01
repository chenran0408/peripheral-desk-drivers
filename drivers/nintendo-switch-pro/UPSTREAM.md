# Nintendo Switch Pro Controller / Joy-Con

- [HaloBattery Nintendo provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/nintendo.py), MIT; referenced for Bluetooth `057E:2009/2006/2007`, full input reports `30/31` and subcommand reply `21`, byte 2 level/charging, and read-only device info subcommand `02`. The controller mode is never changed; USB is excluded.
- [SDL Switch HIDAPI driver](https://github.com/libsdl-org/SDL/blob/main/src/joystick/hidapi/SDL_hidapi_switch.c), Zlib; and [Nintendo Switch protocol notes](https://github.com/dekuNukem/Nintendo_Switch_Reverse_Engineering), MIT; protocol reference only.
- Code-adapter-only; HaloBattery lists these controllers as unverified and Peripheral Desk has not tested hardware.
