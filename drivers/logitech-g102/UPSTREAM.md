# 罗技 G102

基于 [Voltide v2.1.2](https://github.com/sdparklab/Voltide/releases/tag/v2.1.2)，固定提交 `73f16fd2be44570c0a861cef6b4804d743656893` 的 `LogitechProvider.cs`。Voltide 将 `罗技 G102` 记录为真实设备可用，连接范围为 USB/接收器、接收器 PID `046D:C539`，使用 HID++ 2.0 电量特性。

Peripheral Desk 复用协议事实并独立实现 Rust 适配；本项目未独立实测。该插件仅读取电量和充电状态，不声明 DPI、回报率、改键、宏、灯效或其他控制功能。
