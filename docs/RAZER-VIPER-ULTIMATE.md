# Razer Viper Ultimate Wired Preview

Package: `razer-viper-ultimate-wired-driver` (`0.4.6-preview.1`). This independent adapter uses public OpenRazer protocol facts and has not been tested with a physical Viper Ultimate.

## Exact scope

- Device: Razer Viper Ultimate connected by USB cable.
- Exact USB identity: VID/PID `1532:007A`. The wireless model uses PID `007B` and is excluded.
- The adapter opens only a unique interface matching that PID, the `MI_00` path segment, and a 91-byte Feature report. It sends no request when zero or multiple interfaces match. OpenRazer's report structure is 90 bytes; the 91-byte Windows HID feature buffer includes the leading Report ID byte used by this repository's HID transport. No physical Viper Ultimate descriptor was captured, so these interface and buffer guards remain unverified.
- Reads: battery level and charging status. No DPI, polling rate, buttons, macros, lighting, pairing, firmware, or other writes are exposed.
- The adapter accepts only complete status `0x02` replies, checks report ID, checksum, command, and transaction fields, and retries busy/incomplete replies up to five times.

## Protocol evidence

The source was checked at immutable OpenRazer commit [`477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`](https://github.com/openrazer/openrazer/tree/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d). `mouse.py` defines `RazerViperUltimateWired` with VID `0x1532`, PID `0x007A`, and lists `get_battery` and `is_charging` among its methods ([lines 744–760](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py#L744-L760)). `razermouse_driver.h` maps wired and wireless models to `0x007A` and `0x007B` respectively ([lines 58–60](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L58-L60)).

For this model, `razermouse_driver.c` sends and receives with USB control report index `0` and uses `RAZER_VIPER_MOUSE_RECEIVER_WAIT_US` ([lines 32–33 and 87–98](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L32-L33)). That wait is `59900` microseconds ([line 133](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L133)). OpenRazer sets transaction ID `0xFF` for Viper Ultimate battery and charging reads ([battery lines 1492–1585](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1492-L1585), [charging lines 1592–1679](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1592-L1679)).

Battery uses command class `0x07`, command `0x80`, and data size `2`; its value is returned in argument byte 1. Charging uses class `0x07`, command `0x84`, data size `2`; its result is 0 or 1 in argument byte 1 ([`razerchromacommon.c` lines 968–986](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razerchromacommon.c#L968-L986), [battery result lines 1578–1585](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1578-L1585), [charging result lines 1648–1650](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1648-L1650)). The wire report is 90 bytes with 80 argument bytes ([`razercommon.h` lines 114–126](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.h#L114-L126)); its checksum XORs report bytes 2 through 87 ([`razercommon.c` lines 98–113](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.c#L98-L113)).

OpenRazer is licensed GPL-2.0-or-later. This plugin copies no OpenRazer source; the Rust codec independently implements the public report facts above. This repository's AGPL-3.0-only license applies to the plugin and its protocol crate.

## Compatibility

The package uses driver ABI 1 and declares host compatibility `0.4.0` through versions earlier than `0.5.0`. Physical-device behavior remains unverified.
