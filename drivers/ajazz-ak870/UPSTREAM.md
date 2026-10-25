# AJAZZ AK870

- Exact receiver interface: `3151:4011`, usage page `FFFF`, usage `0002`, Feature report length 65.
- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `AjazzCodec.cs`, `AjazzPlugin.cs`, `docs/hardware/ak870-ml903.md` §1. Vigil Watt records real-device validation by contributor KNA7022.
- Query uses Feature report ID `00`, payload `F7 ... 08`; response percent is `[2]`, online flag `[4]` is 0, and `[7]` must be 1 or 3 to confirm keyboard identity. The reported charging state is unknown.
- Protocol was implemented independently from documented facts; no source code copied. Peripheral Desk has not tested this adapter. This package does not expose key mapping or lighting controls.
