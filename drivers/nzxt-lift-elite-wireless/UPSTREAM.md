# NZXT Lift Elite Wireless protocol and scope

- [HaloBattery protocol notes](https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md#nzxt-lift-elite-wireless-1e712101) and [provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/nzxt.py), MIT; license text: [`../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt`](../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt).
- Upstream issue #148 reporter captured CAM 4.76.5 traffic and verified receiver `1E71:2101` and cable `1E71:2129` on hardware. FFCA:0001 uses 64-byte interrupt OUT `4E 02 81 00 B0`, ignores `4E E5` ACK, and reads `4E 02 97` percent LE bytes 17-18 and charging bit 0 of byte 22.
- Independently implemented from these protocol facts. Peripheral Desk has not tested hardware. NZXT CAM concurrently running may prevent the receiver replying.
