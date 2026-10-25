# Razer BlackShark V2 Pro (2023)

Vigil Watt v1.3.0 exact scope: receiver `1532:0555`, `FF00:0001`, `MI_03&Col02`, 64-byte input/output PA audio interface. Protocol fields and the short remote-mode session follow `RazerHeadsetCodec.cs` / `RazerPlugin.cs` at commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`; MIT notice is retained with this driver. The protocol was also cross-referenced upstream to the open OpenRazer PR #2862 (GPL-2.0-or-later); no OpenRazer code was copied.

Vigil Watt records a contributor-validated 79% and charging-status response for the exact receiver. Peripheral Desk has not tested hardware. This plugin sends only remote-mode on/off and the documented battery/charge queries, and must be explicitly enabled. EQ, volume, microphone, firmware, and other writes are not implemented; 2020 BlackShark `0528` and unverified `0556` are excluded.
