# VGN Dragonfly F2 Master+ preview

This is an unverified preview for the VGN Dragonfly F2 Master+. It reports
battery, charging state, firmware version, current DPI stages, and polling
rate. It can change the active DPI stage, polling rate, and supported actions
for the six button slots. Macro upload and lighting control are unavailable.
The button table is present in the upstream implementation, but the source
notes that VGN vendor software has not been captured for this table; use this
feature as an explicitly untested preview.

## Supported USB collections

The fixed OpenMouse `mouse-protocol` revision explicitly lists both F2 Master+
paths in its VGN WebHID driver and support test:

- Wireless receiver: `3554:FB56`
- Wired: `3554:FB57`
- HID collection: usage page `0xFF02`
- Report ID: `0x08`; 16-byte protocol payload, 17-byte Windows HID buffer
  including the report-ID byte

The adapter requires exactly one matching collection with 17-byte input and
output reports. Generic or empty product strings do not change the PID-based
source binding; an explicit product string naming F1 Pro or F2 Pro Max is
refused before any query. It does not match by vendor ID alone and does not accept F1 Pro
(`F502/F503`), F2 Pro Max (`FB3D/FB3E`), or other F2 variants.

## Read protocol and checks

Battery `0x04`, firmware `0x12`, flash read `0x08`, and flash write `0x07` use
the report-08 checksum. Reads validate the exact returned address and length.
DPI is limited to 50–26,000 in 50-DPI steps; rates are limited to 125, 250,
500, 1,000, 2,000, 4,000, or 8,000 Hz. A setting write snapshots its original
bytes, changes only the requested field, checks the acknowledgement and exact
read-back, and attempts to restore and confirm the original bytes on failure.
Each button update writes one four-byte record only, confirms that record by
reading it back, and restores its original bytes if confirmation fails. The
primary left button is fixed, unknown or corrupt records remain read-only, and
at least one left-click assignment must remain.

The interrupt exchange is serialized and bounded to 700 ms. Timed-out Windows
I/O is cancelled and drained by the shared HID transport before its buffers
are released. No reset, pairing, firmware, macro, or lighting operation is
exposed.

OpenMouse's protocol tests include captured-shaped fixture frames, but those
fixtures are not proof of testing on this Windows plugin or on a physical F2
Master+. This preview remains marked unverified until hardware readings and
write retention are checked on the device. The button labels follow the
upstream six-slot order: left, right, middle, back, forward, DPI.

## Sources and license

Protocol facts and model mapping were checked against OpenMouse-Project's
[`mouse-protocol` at `bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc), specifically its
[VGN codec](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/vgn/index.ts),
[VGN HID driver](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/hid.ts), and
[protocol tests](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/protocol.test.ts).
The shared six-record remapping layout is documented in OpenMouse's
[Teevolution codec](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/teevolution/index.ts),
which names VGN F2 among supported consumers. The driver source separately
notes that no VGN vendor-software key-table capture was available, so this
feature remains an unverified preview.
The source repository declares AGPL-3.0-or-later and its `LICENSE` is retained
in the source repository. This Rust adapter is a separate implementation of
the documented packet facts; it does not copy the upstream codec or driver
code. OpenMouse-Project and its contributors retain copyright in their source.

DPI 档位按钮也可切换当前档位；先验证完整档位结构，再只写入活动档位与校验字节，失败时恢复原字节。
