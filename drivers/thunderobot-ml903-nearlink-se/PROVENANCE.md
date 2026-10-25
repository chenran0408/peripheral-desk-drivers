# THUNDEROBOT ML903 NearLink Se

- Exact NearLink receiver: `369B:0234`, usage page `FFA0`, usage `0002`, 64-byte input/output reports.
- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `ThunderobotCodec.cs`, `ThunderobotPlugin.cs`, `docs/hardware/ak870-ml903.md` §2. The source records real-device validation by contributor KNA7022; Peripheral Desk has not tested it.
- Output query is report ID `A0`, command `02`. The matching input must start `A1 02` and contain an unambiguous TLV stream. Battery is tag `0225` with one-byte integer value 0–100; `0226` is workmode and is never interpreted as charging.
- Protocol facts were independently implemented from a public hardware analysis of the official application; no vendor binary or source code is included. No charging state, DPI, macros, remapping, or lighting controls are claimed.
