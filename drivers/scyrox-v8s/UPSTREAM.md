# Scyrox V8S

- Protocol facts: [pedroolmartins/mouse-profile-manager `src/hid/constants.js`](https://github.com/pedroolmartins/mouse-profile-manager/blob/main/src/hid/constants.js), [`src/hid/protocol.js`](https://github.com/pedroolmartins/mouse-profile-manager/blob/main/src/hid/protocol.js), and [`src/stores/mouse.js`](https://github.com/pedroolmartins/mouse-profile-manager/blob/main/src/stores/mouse.js), MIT licensed.
- Identity: wired `3554:F515`, receiver `3554:F517`; 17-byte report ID 08 protocol, battery command 04 and response fields at bytes 6-9. The project implements battery reads for V8S, but does not publish a hardware validation statement for this exact model.
- Status is code-adapter-only. This adapter is independently implemented from protocol facts; no upstream source code is copied. Peripheral Desk has not tested hardware.
