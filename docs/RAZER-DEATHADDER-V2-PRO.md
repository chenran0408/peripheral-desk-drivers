# Razer DeathAdder V2 Pro Wired Preview

Package: `razer-deathadder-v2-pro-wired-driver` (`0.4.7-preview.1`). This independent adapter reads battery level and charging state. Its protocol facts come from OpenRazer; this package has not been locally tested with a physical device.

## Supported connection

- Wired USB model only: VID/PID `1532:007C`. The wireless model uses PID `007D` and is excluded.
- The adapter opens only a unique HID interface matching PID `007C`, the `MI_00` path segment, and a 91-byte Feature report. OpenRazer describes a 90-byte wire report; this repository's Windows HID transport includes a leading Report ID byte in its 91-byte feature buffer. No physical descriptor was captured, so these interface and buffer guards are not locally confirmed.
- Reads only battery level and charging status. DPI, buttons, macros, lighting, pairing, firmware and other writes are not exposed.

## Upstream support evidence

The source was checked at immutable OpenRazer commit [`477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`](https://github.com/openrazer/openrazer/tree/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d). Its `mouse.py` defines `RazerDeathAdderV2ProWired`, assigns VID `0x1532` and PID `0x007C`, and lists both `get_battery` and `is_charging` ([lines 1249–1270](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py#L1249-L1270)). The driver header maps the wired product to `0x007C`, distinct from wireless `0x007D` ([lines 63–64](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L63-L64)).

For this model the driver uses control report index 0 for send and receive, with the `RAZER_VIPER_MOUSE_RECEIVER_WAIT_US` delay ([`razermouse_driver.c` lines 33–103](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L33-L103)); the delay is 59,900 microseconds ([`razermouse_driver.h` line 137](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L137)). OpenRazer sets transaction ID `0x3F` for both battery ([lines 1590–1613](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1590-L1613)) and charging ([lines 1695–1733](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1695-L1733)). The command constructors use class `0x07` with commands `0x80` and `0x84` ([`razerchromacommon.c` lines 1057–1074](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razerchromacommon.c#L1057-L1074)); battery is reported on a 0–255 scale and charging as 0 or 1 ([`razermouse_driver.c` lines 1589–1687 and 1694–1794](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1589-L1687)). The report structure is 90 bytes ([`razercommon.h` line 128](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.h#L128)); the checksum is calculated by XOR ([`razercommon.c` lines 111–124](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.c#L111-L124)).

## Implementation and validation status

The Rust protocol crate implements the public report fields independently; it does not copy OpenRazer source. OpenRazer is GPL-2.0-or-later. The plugin follows this repository's AGPL-3.0-only license. Upstream support is evidence from the cited source; hardware commands and physical-device verification were not performed for this package. Software integration is checked in release CI.

The package uses driver ABI 1 and supports host versions `0.4.0` through versions earlier than `0.5.0`.
