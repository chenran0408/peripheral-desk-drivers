# ATK F1 Pro Max provenance

- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `docs/hardware/atk.md` and `docs/hardware/protocols.md`: F1 Pro Max receiver battery query, 17-byte report `08 04 ...`, battery percent at report byte 6, charging byte 7, voltage bytes 8-9; real-device validation covers receiver PIDs `373B:101A` and `373B:101B` and wired body `373B:1015`.
- OpenMouse `mouse-protocol` commit `23f5bed75d148cabeed19c74838deb31d3f60d02`, ATK WebHID implementation and generated catalog: exact F1 Pro Max body `373B:1015`, CID/MID `2,37`, PAW3950 sensor; EEPROM DPI and polling settings share the ATK report-08 transport. DPI and rate writes below are adapted from those protocol facts/implementation.
- OpenMouse is AGPL-3.0-or-later. This driver is distributed under the same license and the upstream license text is included at `THIRD_PARTY_LICENSES/AGPL-3.0.txt`.

## Exact coverage

Supported battery connections: the two documented F1 Pro Max receivers (`101A`, `101B`) and the documented wired body (`1015`), only on usage page `FF02`, usage `0002`, MI_01, and the report shape described in Vigil Watt. Receiver report length is exactly 17 bytes; wired body must expose at least 17 bytes. The F1 V3 Ultimate+ IDs and other ATK models are intentionally excluded.

Settings implemented from the OpenMouse F1/ATK protocol family: read/write current DPI stage and polling rate, after checking the device CID/MID. DPI/rate EEPROM bytes are read before writing and read back; a failed or mismatched write triggers restoration and verification. 8K polling is exposed only on the 8K receiver PID; wired mode is capped at 1K. Buttons, macros and lighting are not exposed because the referenced F1 profile does not provide a model-specific verified implementation for them. No Peripheral Desk physical-device test has been performed; Vigil Watt's battery validation is upstream evidence, not a Peripheral Desk validation.
