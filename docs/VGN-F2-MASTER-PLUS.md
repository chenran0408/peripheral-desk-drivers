# VGN Dragonfly F2 Master+ preview

This is an unverified, read-only preview for the VGN Dragonfly F2 Master+.
It reports battery percentage, charging state, and the public firmware version
when those replies pass validation. No configuration, button mapping, DPI,
polling-rate, macro, or lighting commands are sent.

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

The only commands sent are battery `0x04` and firmware `0x12`, both with the
report-08 checksum. A response must be exactly 16 payload bytes, have a valid
checksum, echo the requested opcode, and report status zero. Battery replies
must contain a percentage from 0 through 100 and charge state 0 (discharging)
or 1 (charging); all other values fail closed. The interrupt exchange is
serialized and bounded to 700 ms. Timed-out Windows I/O is cancelled and
drained by the shared HID transport before its buffers are released.

OpenMouse's protocol tests include captured-shaped fixture frames, but those
fixtures are not proof of testing on this Windows plugin or on a physical F2
Master+. This preview remains marked unverified until hardware readings are
checked against the device.

## Sources and license

Protocol facts and model mapping were checked against OpenMouse-Project's
[`mouse-protocol` at `bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc), specifically its
[VGN codec](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/vgn/index.ts),
[VGN HID driver](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/hid.ts), and
[protocol tests](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/protocol.test.ts).
The source repository declares AGPL-3.0-or-later and its `LICENSE` is retained
in the source repository. This Rust adapter is a separate implementation of
the documented packet facts; it does not copy the upstream codec or driver
code. OpenMouse-Project and its contributors retain copyright in their source.
