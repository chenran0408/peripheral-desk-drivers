# SteelSeries Arctis Nova Elite protocol and scope

- [HaloBattery protocol notes](https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md#steelseries-arctis-nova-elite) and [provider](https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries_elite.py), MIT; license text: [`../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt`](../../THIRD_PARTY_LICENSES/HaloBattery-MIT.txt).
- Upstream issue #138 reporter checked station `1038:2244` against SteelSeries GG, including charging. Interface 3 64-byte output `01 B0` produces direct `01 B0` (percent byte 6, power byte 14, charge byte 15), or `07 B7` (percent byte 2, charge byte 4) plus `07 B5` (power byte 4). Windows separates report 01 and 07 across HID collections.
- Independent Rust implementation. Peripheral Desk has not tested hardware. Spare station battery and other product IDs are outside scope.
