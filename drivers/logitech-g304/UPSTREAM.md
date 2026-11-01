# 罗技 G304

本插件依据 Voltide v2.1.2（固定提交 `73f16fd2be44570c0a861cef6b4804d743656893`）的 Logitech provider 型号映射，支持 `046D:C53F` LIGHTSPEED 接收器的只读电量读取。该接收器 PID 也被 G305 使用，因此运行时还必须匹配 HID++ 4.2、WPID `4074` 和完整设备名 `G304 Lightspeed Wireless Gaming Mouse`；G305 会被拒绝。

Solaar 的固定 G304 设备记录确认了上述完整名称、WPID、HID++ 4.2 和 `0x1000` 电池特性。Peripheral Desk 尚未独立实测本设备。插件不提供 DPI、回报率、改键、宏、灯光、配对或固件操作。

来源：

- [Voltide v2.1.2 LogitechProvider.cs](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/LogitechProvider.cs)
- [Solaar G304 HID++ 记录](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/docs/devices/G304%20Lightspeed%20Wireless%20Gaming%20Mouse%204074.txt)
- [Voltide MIT License](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/LICENSE)
