# Razer Naga Pro (wireless)

- [OpenRazer device ID](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h) and [battery command handling](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c): `1532:0090`, Feature report 91 bytes, battery class `07`, command `80`, transaction `1F`; raw level 0–255. OpenRazer is GPL-2.0-or-later. Only protocol facts were referenced; no GPL source was copied.
- Receiver PID is specific to this model. Wired `008F`, HyperPolling dongles, and Bluetooth are outside this package.
- Code adapter only: the upstream driver supports battery reporting, but no model-specific hardware reading or Peripheral Desk hardware test is recorded.
