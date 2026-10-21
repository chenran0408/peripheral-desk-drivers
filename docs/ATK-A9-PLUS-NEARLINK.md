# ATK A9 Plus Nearlink

`atk-a9-plus-nearlink` is a read-only preview for the USB wired mouse
(`373B:1115`) and NearLink receiver (`373B:10C9`). Both use the `FF02:0002`
collection and report ID `08` with a 16-byte payload. The Windows HID buffer
includes the report ID, so the driver selects a 17-byte input/output report on
the `MI_01` interface. Other ATK product IDs and the separate `FF04` and
`FF05` collections are outside this driver.

The driver sends identity command `10` first. The captured reply declares a
two-byte payload containing CID/MID `02:53` (decimal `2,83`). Only an exact,
valid identity reply allows battery command `04`. Its reply declares exactly
two payload bytes: percentage (0–100) and charging (`00` or `01`). Bytes after
those two declared values, including `10 8e` in the captures, are padding and
are not interpreted as voltage. No setting, EEPROM, pairing, lighting, or
other commands are sent. Missing or unknown CID/MID and malformed replies stop
the read.

The protocol is independently implemented from the published hardware
records; no OpenMouse source is copied. Evidence is pinned to OpenMouse
`mouse-protocol` commit
[`bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc).
Its [`atk-testing.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/atk-testing.md)
records A9 Plus Nearlink on the wired and receiver paths, mouse firmware 1.20
and 1.30, and receiver firmware 1.20 and 1.23. The raw report-08 traces are
[`wired.json`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/captures/atk-a9-plus-nk/wired.json),
[`receiver.json`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/captures/atk-a9-plus-nk/receiver.json),
[`wired-1.30.json`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/captures/atk-a9-plus-nk/wired-1.30.json),
and
[`receiver-1.30.json`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/captures/atk-a9-plus-nk/receiver-1.30.json).
Across all four captures, command `10` returns status `00`, declared length
`02`, and CID/MID `02 53`; command `04` returns status `00`, declared length
`02`, and 100% with charging on the cable or discharging through the receiver.
The model is identified by runtime CID/MID on both USB PIDs, not by product
string.

These records document upstream real-hardware reads. This driver build has
not been validated locally with hardware.
