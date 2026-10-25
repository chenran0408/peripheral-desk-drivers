# ROG Strix Scope II 96 Omni

- Vigil Watt v1.3.0, commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`.
- Exact path: receiver `0B05:1ACE`, paired Scope II 96 keyboard endpoint `FF00:0001`, 64-byte input/output, report ID 2, path collection `MI_02&COL02`.
- Reads only after the receiver's paired-device report confirms a supported keyboard product ID. Battery byte `[6]`; charge flag `[9]`.
- Vigil Watt explicitly documents an upstream hardware reading on the paired Omni connection. Peripheral Desk has not tested this hardware.
- The protocol implementation here is independently authored from the public specification and is not copied source. Reference source: `src/VigilWatt.Core/Plugins/Asus/{AsusKeyboardCodec.cs,AsusPlugin.cs}` at the commit above.
- Controls are deliberately not declared: Vigil Watt implements battery only. OpenRGB's upstream Scope II 96 report says 2.4 GHz Omni RGB control is not supported; its USB-mode support does not establish Omni support.
