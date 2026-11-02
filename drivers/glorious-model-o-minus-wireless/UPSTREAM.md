# Glorious Model O- Wireless

- Source: [korkje/mow](https://github.com/korkje/mow), `src/util/devices.rs` (258A:2024 receiver, 258A:2013 cable) and `src/report/battery.rs` (65-byte feature report ID 0, 02 02 83 query, percent at byte 8). MIT, copyright 2023 Ola Naess Kaldestad. Protocol facts were independently implemented; no source code copied.
- Upstream implements battery support for this PID pair; no model-specific hardware reading was found. Peripheral Desk has not tested hardware. Charging is deliberately not inferred from cable presence.
