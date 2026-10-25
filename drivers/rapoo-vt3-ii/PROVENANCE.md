# Rapoo VT3 二代

- Exact adapter: 2.4 GHz receiver `24AE:1412`; status endpoint has usage page or usage `FF00` and exactly 19-byte input reports. This PID is not listed in the cited third-party model table; Vigil Watt records its own device as verified and distinguishes the second-generation protocol by the report-7 signature.
- Vigil Watt v1.3.0 (`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`), `RapooCodec.cs`, `RapooPlugin.cs`, `docs/hardware/rapoo.md` §1. Its read is passive: the receiver emits report `07` heartbeats while the mouse is active; no bytes are sent.
- Percent is report `[8]` (1–100); charging flag `[7]` is 1=discharging, 2=charging. Other states and out-of-range percentages are left unknown/ignored. The battery read can wait up to five seconds for a fresh heartbeat; sleep silence is normal.
- Protocol adapter is independently authored from public protocol facts, no source code copied. Peripheral Desk has not tested this hardware. DPI/profile/lighting writes are not exposed in this battery plugin.
