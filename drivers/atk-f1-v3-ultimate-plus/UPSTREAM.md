# ATK F1 V3 ULTIMATE+ battery adapter

- Upstream: Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`; protocol and implementation notes: [`docs/hardware/atk.md`](https://github.com/evren-oct/vigil-watt/blob/a78e973b4eb634bdce37c9078dd48d1c07e3eb73/docs/hardware/atk.md), shared op-frame implementation `AtkCodec.cs` / `AtkPlugin.cs`.
- **Maintainer real-device evidence:** F1 V3 ULTIMATE+ 8K-M receiver `373B:1303`, `MI_01&COL05`, usage `FF02:0002`, 17-byte input and output. Upstream recorded 75%, discharging, 4154 mV. This evidence is limited to that receiver and link.
- Upstream also records a wired body `373B:12A9`, product string `ATK F1 V3 ULTIMATE+`, with a 75%, charging, 4322 mV response. Its exact report lengths were not measured; this adapter requires at least 17-byte input/output and matches the exact product string.
- Only the battery query/percentage, charge flag, and reported voltage are exposed. No control feature is implemented. No Peripheral Desk physical-device verification is claimed.
- Adapted code is in the shared `protocol-atk-f1-battery` crate. MIT license and attribution are included in its `VIGIL-WATT-MIT.txt` file and the packaged output.
