# Razer Kraken V4

Read-only battery and charging query for the 1532:056C USB receiver, report `FF14:0001` with 64-byte report 0x02. Battery and charge use separate receiver commands, so each call alternates one query. Vigil Watt v1.3.0 describes contributor reverse engineering and hardware verification; its maintainer did not independently retest. No EQ, lighting, or headset controls are exposed. Source and MIT attribution are in the protocol crate.
