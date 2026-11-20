# 炼狱蝰蛇的板载指针设置

在外设盒的「插件管理」选择对应型号的预览版插件，进入「设备调节 → 鼠标型号 → 指针与性能」。可分别编辑每个档位的横向和纵向 DPI，也可以单独修改当前没有使用的档位。设置保存在鼠标上。

| 型号 | USB VID/PID 和路径 | 新增设置 | 验证范围 |
| --- | --- | --- | --- |
| DeathAdder V3 Pro | 1532:00B6 有线、1532:00B7 标准接收器 | 在原有调节和改键功能上增加逐档 X/Y 独立编辑。最高 30,000 DPI | 标准接收器此前在本项目实测电量、DPI、回报率及休眠。本次分轴写入用模拟档位表校验 |
| DeathAdder V3 Pro HyperPolling | 1532:00C3 接收器 | 逐档 X/Y、活动档位、125 / 500 / 1000 / 2000 / 4000 / 8000 Hz | Voltide 实物支持记录涵盖电量。新增设置依据 OpenSynapse 对此 PID 的实现 |
| DeathAdder V3 HyperSpeed | 1532:00C5 接收器 | 逐档 X/Y、活动档位、125 / 500 / 1000 Hz。最高 26,000 DPI | Voltide 实物支持记录涵盖电量。新增设置依据 OpenSynapse 对此 PID 的实现 |

三个插件匹配上述连接的 91 字节 Feature 接口（MI_00）。「指针与性能」每次修改先读回板载表，只改所选轴并保留活动档位、其他档位、记录编号和附加字节。回报率通过该型号自己的命令和编码设置。写入后会读回，失败时尝试恢复并报告结果。

标准版鼠标的快捷 DPI 滑块也读取同一份板载表。两轴原本相同时，滑块同步调整两轴；已经分轴时，滑块只调整 X。切换档位会按该档真实的 X、Y 分别生效。

00C3 是 OpenSynapse 设备表单列的 DeathAdder V3 Pro HyperPolling 无线型号，设备包也把 00C3 对应到 00C2 型号定义。通用 HyperPolling Wireless Dongle 在同一设备表里是 00B3。本批插件不匹配 00B3。[上游设备与 PID 对照](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/docs/SUPPORTED_DEVICES.md) · [设备包型号对应](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/src/OpenSynapse.Windows/Devices/RazerDriverPackages.json)

## 资料范围

- 设备目录和协议参考固定在 OpenSynapse v1.7.2，提交 `5b3e6516492c95eb363fbda407a5ee965367ff08`。项目列出 389 个可识别的设备 ID，逐型号功能和验证范围各异。设备能力总表、鼠标指令表、分轴字段和回报率分别来自以下来源。[设备清单](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/docs/SUPPORTED_DEVICES.md) · [按 PID 划分的能力表](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/src/OpenSynapse.Windows/Devices/RazerHid/mice.json) · [写入格式](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/src/OpenSynapse.Windows/Protocols/RazerHidMouseProtocol.cs) · [回复解析](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/src/OpenSynapse.Windows/Protocols/RazerHidMouseProtocol.Parsing.cs) · [单板载配置型号](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/src/OpenSynapse.Windows/Devices/RazerHidDeviceService.OnboardProfiles.cs)
- OpenSynapse 版权所有 © 2026 A1mAssist，授权为 AGPL-3.0-only。外设盒参照其协议事实，在自己的 Rust Feature 传输及 ABI 1 插件上实现。此批分发外设盒的代码与原有设备图，不包含 OpenSynapse 源码、厂商二进制、图像或网页。[OpenSynapse 许可证](https://github.com/A1mAssist/OpenSynapse/blob/5b3e6516492c95eb363fbda407a5ee965367ff08/LICENSE)
- 标准版 00B6/00B7 保留原插件已经使用的 38 字节 GET 格式；独立 X/Y 字段在公开协议中的位置一致。V3 Pro 的分轴 DPI 上限采用其官方硬件规格的 30,000 DPI。高回报率通道的数据编码不适用于普通接收器。[DeathAdder V3 Pro 官方规格](https://www.razer.com/gaming-mice/razer-deathadder-v3-pro) · [DeathAdder V3 HyperSpeed 官方规格](https://www.razer.com/gaming-mice/razer-deathadder-v3-hyperspeed)
- 雷蛇的 Synapse Web 已公开提供 DeathAdder V4 Pro 的板载设置、DPI 档位和按键分配。本文三台 V3 系列使用本地插件。之前的电量接入范围仍见各插件所列的 Voltide 实物报告。[雷蛇官方说明](https://www.razer.com/blog/razer-deathadder-v4-pro-now-supported-on-razer-synapse-web) · [Voltide 固定版本](https://github.com/sdparklab/Voltide/tree/73f16fd2be44570c0a861cef6b4804d743656893)
