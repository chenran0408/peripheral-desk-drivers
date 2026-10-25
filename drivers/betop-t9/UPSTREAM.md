# BETOP T9 receiver

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `docs/hardware/betop.md` §1.4 and `docs/hardware/protocols.md`.
- Exact scope: `20BC:5073`, passive status stream on HID usage `00FF:0003`, input report ID `04`, 24-byte input. T9 does not answer the battery query used by other BETOP receivers, so this plugin never writes a report.
- Byte 23 is a raw level in 8-point steps up to `0x78` and is converted to nearest whole percent on the documented 0–120 scale. The observed `0x50` frame is an offline placeholder and is not shown as 67%.
- Upstream maintainer device use validates the T9 passive stream. Peripheral Desk has not tested hardware. This adapter is independently written from the published protocol; no external source code is copied.
- Charging flag is not present in this stream. No buttons, macros, lighting, or controller configuration are exposed.
