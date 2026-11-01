# Upstream basis

- Project: [sdparklab/Voltide](https://github.com/sdparklab/Voltide)
- Release: [v2.1.2](https://github.com/sdparklab/Voltide/releases/tag/v2.1.2)
- Pinned revision: [`73f16fd2be44570c0a861cef6b4804d743656893`](https://github.com/sdparklab/Voltide/tree/73f16fd2be44570c0a861cef6b4804d743656893)
- Protocol source: [`src/Providers/ATKA9AiraddProvider.cs`](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKA9AiraddProvider.cs)
- License: MIT, Copyright (c) 2026 Joko; exact license text is included in `THIRD_PARTY_LICENSES/Voltide-LICENSE.txt`.
- The upstream provider credits GitHub contributor `yiwluoq`. Voltide reports real-device verification on 2026-11-01 for ATK A9 Air 大师+ and receiver `373B:1278` using `FF02:0002`, 17-byte input/output reports.

This is an independently written Rust port of the published protocol facts, not copied Voltide source code. The protocol is queried with HID `SetOutputReport`: report ID `08`, command `04`; bytes 1–15 and the final checksum byte sum to `4D` modulo 256. A valid 17-byte reply uses the same report and command, percentage byte 6, charging byte 7 (`00` discharging, `01` charging), millivolts bytes 8–9 big-endian, and checksum total `4D`. Other charging flag values remain unknown. The adapter rejects invalid percentages, malformed lengths, mismatched report/command, and bad checksums.

Peripheral Desk has not independently tested this device. This plugin supports only the exact 2.4 GHz receiver interface above. It does not cover NearLink, wired, Bluetooth, other ATK PIDs, DPI, polling rate, remapping, macros, lighting, or audio features.
