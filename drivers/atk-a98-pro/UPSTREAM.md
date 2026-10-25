# ATK A98 Pro

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `AtkPlugin.cs` / `AtkViaCodec.cs`, `docs/hardware/atk.md` §3.
- Exact receiver/model scope: `373B:1152`, VIA channel `FF60:0061`, 33-byte reports, report ID 0.
- Query uses VIA `get_keyboard_value` on custom id `0x96`, battery key `0x24`; reply returns integer percent and charge flag.
- Vigil Watt lists A98 Pro as real-device verified; Peripheral Desk has not tested it.
- Adapter is independently authored from the documented protocol. Vigil Watt's MIT license and source are recorded in its repository; no code copied.
- Control boundary checked against the upstream `the-via/keyboards` repository and official VIA configuration guidance: the repository has no ATK A98 Pro keyboard definition, and the Vigil Watt ATK source implements only `get_keyboard_value` (`0x02`) on the `0x96/0x24` battery channel. A model definition needs the exact matrix/layout and firmware-backed VIA values. The available source does not specify this A98's matrix geometry, `0x04/0x05` keymap read/write payloads, lighting value IDs/ranges, or macro storage/read-back layout. Therefore the generic VIA transport alone is insufficient evidence to expose writes safely; this plugin remains battery-only until an exact A98 Pro VIA definition/firmware implementation is available.
