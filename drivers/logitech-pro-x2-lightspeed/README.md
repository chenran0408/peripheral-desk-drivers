# Logitech PRO X 2 LIGHTSPEED

Vigil Watt v1.3.0 exact support scope: `046D:0AF7`, `FFA0:0001`, 64-byte IN/OUT, 2.4 GHz receiver. Query/report markers, raw integer percent offset, and known charging values are adapted from `CenturionCodec.cs` at commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73` under MIT; the license text is retained beside the protocol crate. The source record references HeadsetControl and LogitechBatteryMonitor but says no GPL code was copied.

Upstream reported receiver real-device readings of 45% discharging and 43% charging. Peripheral Desk has not tested this hardware. The plugin sends the documented read-only battery query; charging marker values beyond `0x00` and `0x02` remain unknown. Bluetooth and USB-wired modes are not included. This adaptation does not implement G HUB EQ, surround, microphone, volume, or other control functions; no compatible open implementation was identified in the upstream source record.
