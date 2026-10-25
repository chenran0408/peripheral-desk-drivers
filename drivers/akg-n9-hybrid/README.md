# AKG N9 Hybrid

Read-only battery query for the exact 0ECB:20D9 USB receiver interface (`MI_03&Col02`, FF13). Percent is taken from the raw result byte and accepted only after the matching successful query acknowledgement. Charging state, EQ, lighting, and other controls are not implemented. Protocol source and MIT attribution: Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`; see the protocol crate's `VIGIL-WATT-MIT.txt`.
