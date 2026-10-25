# HyperX Cloud Flight 2 (飓风炫彩版)

Passive battery, online, and charging-state listener for the exact 03F0:0AC1 dongle status interface `MI_03&COL04`, usage `FF90:FF00`. It does not send any HID commands. Battery is the integer byte at offset 3 of `FB 13 00` events; charging comes only from `FB 09 00/01`. Other Cloud Flight generations and wired/Bluetooth paths are out of scope. Source: Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`; MIT attribution is in `VIGIL-WATT-MIT.txt`. Audio/EQ, lighting, and headset controls are not implemented.
