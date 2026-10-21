# MCHOSE A7 V3 Ultra+ Receiver Preview

Package: `mchose-a7-v3-ultra-plus` (`0.4.4-preview.1`). This adapter has not been tested on this project’s hardware. The fixed upstream protocol notes include a 2026-09-12 diagnostic capture from an A7 V3 Ultra+ on its 8 kHz receiver.

## Supported scope

- Receiver only: USB `3837:1018`, HID usage `0xff01:0x0001`, 64-byte input and output reports, output report ID `0x4d`.
- The host HID product string must equal `MCHOSE A7 V3 Ultra+`. The adapter then sends only the read-only `0x0900` identity/power request.
- The response must have the expected report ID, protocol version, checksum flag and XOR checksum, matching command and transaction fields, and exactly 16 identity bytes. Runtime identity must report vendor `0x3837` and internal product ID `0x4026`.
- The internal `0x4026` does not identify this model by itself: the upstream product table associates it with A5 V3 Ultra+, while the captured A7 V3 Ultra+ receiver reports that value. The exact host product string is therefore required alongside the captured vendor/product pair. If either identity check fails, the adapter returns an error without exposing battery data.
- Battery must be 0–100. Charge status accepts only `0` (not charging) or `1` (charging), following the fixed upstream reader’s interpretation; other values are rejected.
- Link status `0` is documented upstream as the mouse not being reachable; the adapter then withholds battery and charge values. Status `2` is the value in the fixed A7 capture and is accepted as connected. Any other status is rejected because the fixed source does not establish its meaning.
- Firmware is unknown. No settings, buttons, macros, lighting, pairing or other writes are implemented. Wired PID `3837:4033`, other receivers and other MCHOSE models are excluded.

The package image is the project’s original neutral mouse illustration. It is an identification placeholder, not a picture of the A7 V3 Ultra+.

## Protocol and licensing

The reference is OpenMouse Project’s [`mouse-protocol` commit `bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc), specifically [`src/mchose/v3.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/v3.ts), [`src/drivers/mchose/v3-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/v3-hid.ts), and [`docs/mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md). The reference repository is licensed under [GNU AGPL v3](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/LICENSE). This package uses the cited protocol facts and capture as evidence; it contains no copied upstream implementation code. The package remains under this repository’s `AGPL-3.0-only` license.

## Verification status

The source capture supports the receiver identity, report framing and read fields. The implementation is not hardware-verified in this project. A clean protocol fixture test or build does not confirm behavior against a physical mouse or other firmware versions.
