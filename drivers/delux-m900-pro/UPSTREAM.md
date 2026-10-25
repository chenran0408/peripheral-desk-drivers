# DELUX M900 Pro

- Exact passive interface: `1D57:FA65`, usage `000A:0000`, 5-byte input, no output or Feature report. This is distinct from INPHIC ZHEN's `FF00:0001` collection despite the shared VID/PID.
- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `DeluxCodec.cs`, `DeluxPlugin.cs`, `docs/hardware/delux.md` §1. It records contributor real-device evidence, not maintainer retest.
- Passive frame is `03 30 40 <flag> <percent>`; flag `01`=discharging, `03`=charging, `02`=full. Battery is the integer byte `[4]`; values above 100 are ignored.
- This code is independently authored from published protocol facts; no code is copied. Peripheral Desk has not tested it. No polling, remapping, macros, or lighting writes are implemented. Charging percentage can be distorted while dock charging; this stateless adapter reports the value delivered by the upstream frame and does not synthesize/interpolate it.
