# LAMZU Maya X protocol and scope

- [HaloBattery notes](https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md#lamzu-maya-x) and [provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/lamzu.py), MIT; license text: [`../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt`](../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt). Related original exchange: [Sheroune/lamzu-battery-monitory](https://github.com/Sheroune/lamzu-battery-monitory).
- Upstream diagnostic confirms Maya X 8K receiver `373E:001E` on interface 2, FFFF vendor page, report 0 Feature `00 00 02 02 00 83`, reply `A1 00 02 02 00 83` with charge and percent bytes 6-7. Cable ID `373E:001C` is listed upstream but has no separate confirmation.
- Independent Rust implementation. Peripheral Desk has not tested hardware.
