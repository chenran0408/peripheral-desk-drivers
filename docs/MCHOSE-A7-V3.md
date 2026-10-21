# MCHOSE A7 V3 Ultra+ Receiver Preview

Package: `mchose-a7-v3-ultra-plus` (`0.4.9-preview.1`). This is an untested preview: the read and write logic follows the pinned OpenMouse source and captures, and has not been exercised against local hardware. The fixed upstream notes include a 2026-09-12 diagnostic capture from an A7 V3 Ultra+ on its 8 kHz receiver.

## Supported scope

- Receiver only: USB `3837:1018`, HID usage `0xff01:0x0001`, 64-byte input and output reports, output report ID `0x4d`.
- The host HID product string must equal `MCHOSE A7 V3 Ultra+`. Before reading or writing settings, the runtime identity must return vendor `0x3837` and internal product ID `0x4026`; the report framing and XOR checksum are checked.
- The response must have the expected report ID, protocol version, checksum flag and XOR checksum, matching command and transaction fields, and exactly 16 identity bytes. Runtime identity must report vendor `0x3837` and internal product ID `0x4026`.
- The internal `0x4026` does not identify this model by itself: the upstream product table associates it with A5 V3 Ultra+, while the captured A7 V3 Ultra+ receiver reports that value. The exact host product string is therefore required alongside the captured vendor/product pair. If either identity check fails, the adapter returns an error without exposing battery data.
- Battery must be 0–100. Charge status accepts only `0` (not charging) or `1` (charging), following the fixed upstream reader’s interpretation; other values are rejected.
- Link status `0` is documented upstream as the mouse not being reachable; the adapter then withholds battery and charge values. Status `2` is the value in the fixed A7 capture and is accepted as connected. Any other status is rejected because the fixed source does not establish its meaning.
- DPI can be changed on the active stage from 50 to 50,000 in 50-DPI steps. Wireless polling supports 125, 500, 1,000, 2,000, 4,000 and 8,000 Hz. Both use the current profile and preserve the rest of the captured table/settings bytes. Each write is read back; failures attempt to restore the captured original and read it back.
- Six physical buttons are reported and can be reassigned to disabled, mouse buttons or DPI cycling. The remaining five assignments are carried through unchanged; raw unknown assignments are preserved and are not offered as invented choices. Firmware is unknown. Macro writes and lighting are not exposed; this A7 mouse has no controllable onboard lighting in the upstream notes. Pairing and firmware operations are excluded.
- Wired PID `3837:4033`, other receivers and other MCHOSE models are excluded.

The package image is the project’s original neutral mouse illustration. It is an identification placeholder, not a picture of the A7 V3 Ultra+.

## Protocol and licensing

The reference is OpenMouse Project’s [`mouse-protocol` commit `bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc), specifically [`src/mchose/v3.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/v3.ts), [`src/drivers/mchose/v3-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/v3-hid.ts), and [`docs/mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md). The reference repository is licensed under [GNU AGPL v3](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/LICENSE). This package uses the cited protocol facts and capture as evidence; it contains no copied upstream implementation code. The package remains under this repository’s `AGPL-3.0-only` license.

## Verification status

Upstream captures support the receiver identity, settings formats, and button table. The upstream driver explicitly says writes have not been exercised on hardware; this preview shares that limitation. Fixture tests and a successful build do not establish that a physical mouse accepts these writes or that other firmware behaves the same way.
