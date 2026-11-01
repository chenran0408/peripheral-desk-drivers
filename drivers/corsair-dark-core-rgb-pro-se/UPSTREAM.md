# Corsair Dark Core RGB Pro SE protocol and scope

- [HaloBattery protocol notes](https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md#corsair-dark-core-rgb-pro-se) and [provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/corsair.py), MIT; license text: [`../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt`](../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt).
- Hardware confirmation in HaloBattery issue #56: receiver `1B1C:1B7F`, FF42:0001, report ID 0 + 64-byte routed frame to mouse route 09, GET `02`, battery property `0F`. Reply `01 02 00 <LE16>` is tenths of percent. It does not echo property ID, so two equal values are required.
- Wired `1B1C:1B7E` is not covered upstream. Independent Rust implementation; Peripheral Desk has not tested hardware. Charge state is not part of this reply.
