# Logitech G305 preview

This preview reads battery state and supports changing the sensor DPI on an exact G305. It has not been tested on physical hardware in this project. Solaar's pinned upstream G305 record documents HID++ 4.2, receiver WPID `4074`, the exact name `G305 Lightspeed Wireless Gaming Mouse`, Adjustable DPI feature `0x2201` version 1, and Onboard Profiles enabled. DPI stages and current DPI are queried from the device each time; no fixed DPI range is guessed.

The driver only opens the `046D:C53F` receiver's `FF00:0002` 20-byte HID++ interface and always addresses paired slot 1. At runtime it also requires HID++ 4.2, public WPID `4074`, and the exact G305 name. A G304, another paired model, a different receiver, Bluetooth, or a shared receiver is not accepted based on USB identity alone.

Before a DPI write, the driver reads feature `0x8100` and proceeds only when Onboard Profiles reports disabled (`0`). If a profile is active, the operation stops without changing it; turn off Onboard Profiles in Logitech software and retry. The selected DPI must appear in the device's live `0x2201` DPI list. The driver records the current DPI, writes only the requested sensor DPI, then reads it back. If the write or verification fails, it attempts to restore the original DPI and verifies the restoration. A failed rollback is reported explicitly.

Polling rate is not exposed because the upstream G305 record has an active onboard profile that controls polling and buttons; this preview does not alter profiles or infer their layout. Remapping, macros, lighting, pairing, firmware operations, and onboard-profile writes are also unavailable. Battery values can jump between firmware-reported levels rather than changing by one percent at a time.

## Upstream evidence

- [Solaar G305 device record at e7304c4](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/docs/devices/G305%20Lightspeed%20Wireless%20Gaming%20Mouse%204074.text) records the exact model identity, HID++ 4.2, active onboard profiles, `0x2201` V1 and current DPI.
- [Solaar adjustable DPI implementation](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/lib/logitech_receiver/settings_templates.py) describes the DPI list and read/write functions and warns that onboard profiles may override DPI.
- [Solaar onboard-profile and report-rate implementation](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/lib/logitech_receiver/hidpp20.py) and the settings definitions document the profile mode and report-rate feature.

Solaar source is GPL-2.0-or-later; attribution and license are retained in `../THIRD_PARTY_LICENSES/Solaar/NOTICE.txt`. This driver is an independent Rust implementation and has no hardware validation claim.

The raw 0x8100 mode byte must be 0x02 (host mode), not the UI Disabled choice value 0. Mode 0x01 is onboard control; all other values remain unavailable. DPI list requests include sensor 0, direction 0 and page index; zero-terminated entries may end before odd-length reply padding.

0.4.9-preview.2 exposes the main DPI editor (200–12,000, 50-DPI steps) using [Logitech specifications](https://www.logitechg.com/en-us/shop/p/g305-lightspeed-wireless-gaming-mouse.910-006020). The live 0x2201 list still validates every write. Supported sensor values are not presented as onboard profile slots.
