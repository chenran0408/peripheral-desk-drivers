# Pulsar X2 V2 Mini

- Protocol facts: [HaloBattery `docs/protocols.md`](https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md) lists X2 V2 Mini with the Pulsar report-08 family; [python-pulsar-mouse-tool `pulsar.py`](https://github.com/andrewrabert/python-pulsar-mouse-tool/blob/main/pulsar.py) contains the frame implementation (MIT).
- IDs: 2.4 GHz receiver `3554:F508`, USB cable `3554:F507`; report 08, 17 bytes, command 04, battery at byte 6, cable/charging flag at byte 7, voltage at bytes 8-9, checksum byte 16.
- These IDs overlap the existing Pulsar X2 V2 entry and the report does not distinguish the two models. This entry requires manual model confirmation. The upstream implementation does not establish separate hardware validation for the Mini; Peripheral Desk has not tested it. No third-party code was copied.
