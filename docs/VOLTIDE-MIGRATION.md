# Voltide 设备适配映射

参考项目：[sdparklab/Voltide](https://github.com/sdparklab/Voltide)，最新稳定版 `v2.1.2`，固定提交 [`73f16fd2be44570c0a861cef6b4804d743656893`](https://github.com/sdparklab/Voltide/tree/73f16fd2be44570c0a861cef6b4804d743656893)，MIT License。Voltide README 和对应 provider 对目标型号明确记载了真实设备验证；下表保留其型号及连接范围。本项目迁移只表示实现和软件接口已接入，不代表 Peripheral Desk 在本机实物验证。

| Voltide 型号 | 型号/连接证据 | Peripheral Desk 对应项 | 迁移结果与边界 |
|---|---|---|---|
| Logitech G304 | `046D:C53F`，LIGHTSPEED 接收器，HID++ 2.0；WPID `4074`、完整设备名可区分 G304/G305 | [`logitech-g304`](COMPATIBILITY.md) | 已新增独立只读电量插件；与共用接收器 PID 的 G305 分开校验，Voltide/公开设备记录支持，本项目未独立实测。 |
| Logitech G Pro X Superlight 2 | `046D:C547` / `C54D` 接收器；`C09B` 有线 | `logitech-g-pro-x-superlight-2` | 已新增只读电量/充电状态插件；Voltide 上游真机报告，本项目未独立实测。 |
| Logitech M750M | `046D:C548` Logi Bolt | `logitech-m750m` | 已新增精确 PID 只读电量/充电状态插件；不复用 MX Master 3S 身份。 |
| Logitech G102 | `046D:C539`，HID++ 2.0 | `logitech-g102` | 已新增精确 PID 只读电量/充电状态插件；本项目未独立实测。 |
| MCHOSE K99 v2 | `3554:FA09`，MCHOSE 20 字节无线键盘协议 | [`mchose-k99-v2`](COMPATIBILITY.md)，与 G75 共用接收器，手动认领 | Voltide v2.1.2 上游验证范围为 K99 电量；本项目未实机复测。仅提供只读电量/已知充电状态，不提供 G75 设置/改键。接收器无法自动确认型号，须用户手动选择。 |
| MCHOSE G75 Pro | `3554:FA09`，MCHOSE 20 字节无线键盘协议 | [`mchose-g75-pro`](COMPATIBILITY.md) | Peripheral Desk 保留独立设备适配。核心在读写前查询 `0x05`，只接受 G75 Pro 型号签名 `[03 00 00 00 01 15]`；其验证记录与 Voltide 上游贡献者样本分别记录。 |
| ROG Azoth Extreme | `0B05:1ACE` Omni 接收器，FF00/0001、64 字节帧 | [`asus-rog-azoth-extreme`](COMPATIBILITY.md)，与 Scope II 96 共用接收器，手动认领 | Voltide v2.1.2 上游验证范围为 Azoth Omni 电量；本项目未实机复测。仅只读电量/充电，Omni 配对回复不能证明型号，须用户手动选择。 |
| Razer Viper Ultimate 无线 | 接收器 `1532:007B`；91 字节 Feature、`MI_00` | `razer-viper-ultimate-receiver` | 已接入单独只读电量插件；不与有线 `007A` 条目混用。 |
| Voltide Razer 无线 PID 清单 | `00C1,00C3,00BF,00E6,00AB,00CD,00D7,00C5,00D1,00C8,00D4,00B0,00DB,00B4,00B8,00B9,009A,005A,0045`；91 字节 Feature、`MI_00` | 18 个按 PID 区分的插件，见[兼容表](COMPATIBILITY.md) | 只读电量/充电字段；精确按 PID 匹配，转发 Voltide 的事务校验与 `0..100`/`0..255` 百分比换算。不提供其他设置。 |
| Razer BlackShark V2 Pro | `1532:0555`，64 字节音频接收器通道 | [`razer-blackshark-v2-pro-2023`](COMPATIBILITY.md) | 已有 2023 版专用插件；不推广至其他 BlackShark 代际。 |
| ATK A9 Air 大师+ | 接收器 `373B:1278`，`FF02`，17 字节输入/输出；`HidD_SetOutputReport` 查询 | `atk-a9-air-master-plus` | 已新增独立只读电量插件；与 A9 Plus Nearlink (`1115`/`10C9`) 严格分开。Voltide/贡献者报告为真机验证；本项目未实测。 |
| ATK N9 V2 Ultimate | 接收器 `373B:1262`，`FF90`，64 字节输入/输出；`WriteFile` 查询 | `atk-n9-v2-ultimate` | 本批新增独立只读电量/已知充电状态插件；不与同 VID/PID 的 ATK 鼠标接口混淆，限定 `FF90` 集合。本项目未实测。 |

## 许可和实现来源

Voltide 协议事实和设备验证范围固定在 v2.1.2（`73f16fd2be44570c0a861cef6b4804d743656893`）。ATK 适配保留 MIT 许可原文；Logitech/Razer Rust 适配按对应上游 provider 资料独立实现，未复制厂商二进制文件。全部插件仍以本仓库 ABI 1、Rust DLL、设备专属 key 和按需目录发布。Voltide 是电量监控程序，不能作为 DPI、回报率、改键、宏、灯效或音效的证据。

来源：

- [Voltide v2.1.2 发布页](https://github.com/sdparklab/Voltide/releases/tag/v2.1.2)
- [MCHOSE provider（K99/G75 共用 PID 与 20 字节协议）](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/MchoseProvider.cs)
- [ROG provider（Azoth Extreme 的 Omni 状态帧）](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/AsusRogProvider.cs)
- [ATK A9 Air+ provider](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKA9AiraddProvider.cs)
- [ATK N9 V2 Ultimate provider](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKN9V2UltimateProvider.cs)
- [Voltide MIT License](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/LICENSE)
