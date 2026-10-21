# Razer Basilisk V3 Pro Wired Preview

Package: `razer-basilisk-v3-pro-wired` (`0.4.3-preview.1`). This is a public-implementation adaptation and has not been tested on the physical device.

## Exact scope

- Device: Razer Basilisk V3 Pro, USB wired connection only.
- Identity: USB VID/PID `1532:00AA`.
- HID collection: the unique path containing `MI_00` and a 91-byte Feature report. OpenRazer selects report index 0 for this PID. The 91-byte length is the Windows report-ID byte plus the 90-byte protocol structure; this is a matching guard, not a measured Basilisk descriptor. The plugin sends nothing when zero or multiple matching interfaces are found.
- Reads: battery level and charging status only. No DPI, polling-rate, button, lighting, macro, pairing, or firmware writes are available.
- Firmware: unknown. Wireless receiver PID `00AB`, Basilisk V3 Pro 35K variants, Bluetooth, and other Razer products are excluded.
- Reply handling is deliberately conservative: only status `0x02` is accepted as completed. Busy `0x01` is retried up to five queries and then reported as unavailable. OpenRazer accepts busy replies for some commands; this preview does not assume a busy battery reply contains a fresh value without model-specific evidence. HID I/O errors are returned immediately. These differences may reject a device that upstream can read; they do not produce guessed battery values.

## Protocol evidence

OpenRazer was checked at immutable commit [`477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`](https://github.com/openrazer/openrazer/tree/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d). Its `daemon/openrazer_daemon/hardware/mouse.py` defines `RazerBasiliskV3ProWired` with battery and charging reads at [lines 1694–1728](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py#L1694-L1728). `driver/razermouse_driver.h` maps the wired model to PID `0x00AA` at [line 92](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L92). In `driver/razermouse_driver.c`, the model report path uses report index 0 ([lines 35–100](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L35-L100)); the battery query sets TID `0x1F` ([lines 1592–1668](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1592-L1668)) and the charging query also sets TID `0x1F` ([lines 1706–1775](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1706-L1775)). The 31 ms wait constant is defined at [razermouse_driver.h line 135](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L135). The common protocol defines battery (`class 0x07`, command `0x80`) and charging (`class 0x07`, command `0x84`) reads in [razerchromacommon.c](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razerchromacommon.c#L1053-L1070), with a 90-byte report layout and XOR checksum in [razercommon.h](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.h#L124-L136) and [razercommon.c](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razercommon.c#L111-L145).

OpenRazer source is licensed GPL-2.0-or-later. This plugin contains no copied OpenRazer source; its Rust report codec is an independent implementation of the cited public protocol details. The plugin and its protocol crate remain under this repository's `AGPL-3.0-only` license.

## Build and checks

From the driver repository root, build the plugin for Windows x64 with:

```powershell
cargo build --release --target x86_64-pc-windows-msvc -p razer-basilisk-v3-pro-wired-driver
```

The protocol unit tests do not access a mouse:

```powershell
cargo test -p peripheral-desk-protocol-razer
```

Successful compilation and fixture tests check the Rust adapter and report guards only; they do not verify a physical mouse or its firmware.
