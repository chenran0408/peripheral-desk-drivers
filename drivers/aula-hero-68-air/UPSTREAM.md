# AULA HERO 68 Air

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`, `AulaHero68Codec.cs`, `AulaHero68Transport.cs`, `AulaHero68PacketAssembler.cs`, `docs/hardware/aula.md` §2.
- Exact receiver/interface: `372E:106C`, `FF60:0061`, 20-byte reports, report ID `09`; UUID response is checked before battery query.
- Uses the upstream 8K-fragment exchange and matching per-fragment ACKs. Battery response reports integer percent at `[6]` and documented status bits at `[7]`.
- Vigil Watt lists this exact model as real-device verified. Peripheral Desk has not tested it.
- Protocol port is adapted from Vigil Watt's MIT-licensed implementation. Original notice is included in `UPSTREAM-MIT.txt`; modified Rust implementation is in `src/lib.rs`.
- Device controls are not provided by Vigil Watt and are not included here. Other sources' lighting/keymap support remains to be integrated for the exact receiver mode.
