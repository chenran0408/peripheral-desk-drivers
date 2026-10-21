# Razer Viper V2 Pro Wired Preview

Package: `razer-viper-v2-pro-wired-driver` (`0.4.5-preview.1`). This is an independent read-only adaptation of public protocol facts. It has not been tested with a physical Viper V2 Pro.

## Exact scope

- Device: Razer Viper V2 Pro connected by USB cable.
- Exact USB identity: VID/PID `1532:00A5`. The wireless model has a separate PID `00A6` and is excluded.
- The adapter only opens a unique interface matching that PID, the `MI_00` path segment, and a 91-byte Feature report. It sends nothing if zero or multiple interfaces match. The 91-byte Windows HID feature buffer accounts for the report ID plus OpenRazer's 90-byte protocol report. This descriptor length and Windows interface path are guards inferred from the shared report format and repository HID transport conventions; no physical Viper descriptor was captured.
- Reads: battery level and charging status. No DPI, polling rate, button, macro, lighting, pairing, firmware, or other writes are exposed.
- The adapter accepts only complete status `0x02` replies, checks report ID, checksum, command and transaction fields, and retries busy/incomplete replies up to five times. This conservative response handling may report unavailable where OpenRazer retries or accepts a different status; it will not substitute a guessed value.

## Protocol evidence

The source was checked at immutable OpenRazer commit [`477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`](https://github.com/openrazer/openrazer/tree/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d). In `mouse.py`, `RazerViperV2ProWired` is explicitly named as the wired model, sets VID `0x1532` and PID `0x00A5`, and lists `get_battery` and `is_charging` among its supported methods ([lines 1541–1551](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py#L1541-L1551)). `razermouse_driver.h` separately maps the wired constant to `0x00A5` and wireless to `0x00A6` ([lines 88–89](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L88-L89)). The model is in the driver's report-index-zero handling group ([lines 35–55](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L35-L55)).

The model is included in the `0x1F` transaction ID group for both battery and charging queries ([battery lines 1624–1666](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1624-L1666), [charging lines 1740–1775](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1740-L1775)). OpenRazer defines battery as command class `0x07`, command `0x80`, and charging as class `0x07`, command `0x84`; each request declares two data bytes ([razerchromacommon.c lines 1053–1070](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razerchromacommon.c#L1053-L1070)). The report contains an 80-byte argument field and is 90 bytes long ([razercommon.h lines 124–136](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.h#L124-L136)); its XOR checksum covers bytes 2 through 87 ([razercommon.c lines 108–124](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.c#L108-L124)). OpenRazer uses a 31 ms response wait for this report path ([razermouse_driver.h line 135](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L135)).

OpenRazer is licensed GPL-2.0-or-later. This plugin copies no OpenRazer source; the Rust codec independently implements the public report facts above. This repository's AGPL-3.0-only license applies to the plugin and its protocol crate.

## Compatibility

The package uses driver ABI 1 and declares host compatibility `0.4.0` through versions earlier than `0.5.0`. Physical-device behavior remains unverified.
