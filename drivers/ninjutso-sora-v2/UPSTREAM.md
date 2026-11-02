# Ninjutso Sora V2

- Protocol facts: [lllexalllepexa/ninjutso-sora-v2-battery-tray `README.md`](https://github.com/lllexalllepexa/ninjutso-sora-v2-battery-tray/blob/main/README.md) and [`sora_battery.py`](https://github.com/lllexalllepexa/ninjutso-sora-v2-battery-tray/blob/main/sora_battery.py), MIT licensed. The README documents facts extracted from the official NinjaForce web driver and includes example replies.
- Identity: receiver `1915:AE1C`, cable `1915:AE12`, usage page `FFA0`; Feature report 05, 31-byte command payload; command 15 and battery/charge/online bytes 9-12 in the reply.
- The public implementation has not established independent hardware validation. This adapter is independently implemented from protocol facts; no upstream source code is copied. Peripheral Desk has not tested hardware.
