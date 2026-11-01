# ASUS ROG Pugio II

- 0B05:1908 (receiver), 0B05:1906 (USB cable), battery command 12 07 and 0-4 level conversion: https://github.com/seerge/g-helper/blob/main/app/Peripherals/Mouse/Models/PugioII.cs and https://github.com/HeyOkay/HaloBattery/blob/main/providers/asus.py .
- G-Helper is GPL-3.0; HaloBattery is MIT, copyright (c) 2026 Halo Battery contributors. Only protocol facts were consulted. This adapter uses the existing Peripheral Desk ROG protocol crate and copies no upstream code or assets.
- Battery and charging are code-adapter-only, without Peripheral Desk hardware verification.
