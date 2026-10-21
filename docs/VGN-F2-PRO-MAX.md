# VGN Dragonfly F2 Pro Max preview

This unverified preview reads battery percentage and charging state only. It
does not change DPI, polling rate, buttons, macros, lighting, or other settings.
It does not request firmware because the cited model-specific implementation
only establishes the battery transaction for this support entry.

## Supported USB paths

The fixed `quiet-battery` revision lists VGN F2 Pro Max as VID `3554`, receiver
PID `FB3E`, wired PID `FB3D`. It requires vendor usage `FF02:0002` with 17-byte
input and output reports. The adapter matches those IDs
exactly and stops if the product string names another supported VGN model.

The report contains Windows report ID `08` followed by a 16-byte payload. The
only request is command `04` with checksum `49`. The 16-byte response payload
must have command `04`, status `00`, and a valid checksum; percentage is payload
byte 5 (0–100) and charging is payload byte 6 (0 or 1). Unknown charging values,
invalid lengths, wrong commands, bad checksums, and a response that only echoes
the request are rejected. Battery voltage is parsed by the shared codec but is
not exposed as a setting.

## Evidence and limits

`quiet-battery` at commit
[`d6eb4bdade47dccd7cb402e16ac50a6f351fcac0`](https://github.com/TheWebDog/quiet-battery/tree/d6eb4bdade47dccd7cb402e16ac50a6f351fcac0)
explicitly maps this model to the receiver and wired PIDs and implements the
17-byte Compx battery transaction. Its support table describes the new profile
as implemented and awaiting physical confirmation; its published 35% packet
is a shared Compx fixture, not a F2 Pro Max hardware capture. The independent
Rust adapter reuses this repository's VGN checksum and battery parser and does
not copy the `quiet-battery` implementation or assets.

The packet shape agrees with the report-ID/payload split and checksum rule in
OpenMouse's VGN codec at the fixed
[`mouse-protocol` revision](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc).
That revision's WebHID driver only matches F2 Master+ `FB56/FB57` and explicitly
does not establish F2 Pro Max support. Its mapping is not used for this plugin.

The referenced `quiet-battery` source is licensed under PolyForm Noncommercial
1.0.0, which its repository identifies as a noncommercial source license, not
an OSI-approved open-source license. Only model IDs and protocol facts are
referenced here; no upstream code or artwork is copied. This Windows plugin has
not been tested with an F2 Pro Max device.
