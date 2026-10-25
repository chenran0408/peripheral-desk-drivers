# ATK F1 Ultimate 2.0 battery adapter (preview; not device-verified)

- Upstream: Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`; [`docs/hardware/atk.md`](https://github.com/evren-oct/vigil-watt/blob/a78e973b4eb634bdce37c9078dd48d1c07e3eb73/docs/hardware/atk.md), `AtkPlugin.cs`, and `AtkCodec.cs`.
- Vigil Watt maps PID `373B:11D9` and `373B:11E4` to ATK F1 Ultimate 2.0 and routes them through the generic ATK mouse op-frame implementation. Its upstream code applies a VID, `FF02:0002`, and minimum report-length match to these candidates.
- **Evidence limitation:** the cited hardware document has no model-specific captured battery response, measured report lengths, or explicit physical-use report for either PID. The overall ATK validation record refers to other models (F1 Pro Max and F1 V3 ULTIMATE+), which is not evidence for Ultimate 2.0. This adapter is therefore preview/code-adapted only, with no claim of model-specific hardware verification.
- The plugin only sends the shared upstream battery query and parses battery percentage, charge state, and voltage. It is not matched to F1 Pro Max PIDs and implements no control features. Install only for the exact listed PIDs and matching interface.
- Adapted code is in the shared `protocol-atk-f1-battery` crate. MIT license and attribution are included in its `VIGIL-WATT-MIT.txt` file and the packaged output.
