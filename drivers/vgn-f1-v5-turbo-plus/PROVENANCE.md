# VGN F1 V5 Turbo+ provenance

- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `docs/hardware/vgn.md` §1 and `docs/hardware/protocols.md`: contributor real-device evidence for battery and charging on the exact 2.4 GHz receiver `391D:1A23` and wired mouse `391D:1026`. The implementation selects `FFC1:0001`, 64-byte input/output collections, sends report `B3` command `06`, requires response report `B4` echoing `06`, and reads data byte 19 bits 0–6 as percent and bit 7 as charging.
- No third-party source code is copied. Protocol facts and field meanings are attributed to Vigil Watt's documentation, which in turn cites the VGN official `hub.vgnlab.com.cn` web driver bundle.

Vigil Watt's real-device verification covers the battery and charging fields only. The response also carries configuration bytes, but this plugin intentionally does not expose DPI, polling, lighting, remapping, macros, or other controls because their fields and write transactions are not documented/verified for this exact model here. Peripheral Desk has not independently tested this adapter.
