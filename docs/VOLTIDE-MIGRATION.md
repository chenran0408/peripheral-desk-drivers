# Voltide 设备适配映射

参考项目：[sdparklab/Voltide](https://github.com/sdparklab/Voltide)，最新稳定版 `v2.1.2`，固定提交 [`73f16fd2be44570c0a861cef6b4804d743656893`](https://github.com/sdparklab/Voltide/tree/73f16fd2be44570c0a861cef6b4804d743656893)，MIT License。Voltide README 和对应 provider 对目标型号明确记载了真实设备验证；下表保留其型号及连接范围。本项目迁移只表示实现和软件接口已接入，不代表 Peripheral Desk 在本机实物验证。

| Voltide 型号 | 型号/连接证据 | Peripheral Desk 对应项 | 迁移结果与边界 |
|---|---|---|---|
| Logitech G304 | `046D:C53F`，LIGHTSPEED 接收器，HID++ 2.0 | [`logitech-g305`](COMPATIBILITY.md) | 已有独立 G305 插件要求精确产品身份；G304 不因共用 VID/PID 自动复用。 |
| Logitech G Pro X Superlight 2 | `046D:C547` / `C54D` 接收器；`C09B` 有线 | 暂未建立独立型号插件 | HID++ 共用协议不足以代替该型号身份和接收器范围；后续按 PID/配对信息拆分适配。 |
| Logitech M750M | `046D:C548` Logi Bolt | [`logitech-bolt-mx-master-3s`](COMPATIBILITY.md) | 该插件要求识别到配对设备为 MX Master 3S。M750M 不匹配这一型号条件，不扩展其设备名。 |
| Logitech G102 | Voltide 按设备表记载为 HID++ 2.0 | 暂未接入 | 需要明确其具体 USB/接收器 PID 与连接方式后再新增型号条目。 |
| MCHOSE K99 v2 | `3554:FA09`，MCHOSE 20 字节无线键盘协议 | 与 [`mchose-g75-pro`](COMPATIBILITY.md) 存在同号风险 | Voltide 明确指出该 USB 标识也可能属于狼蛛键盘；不能只按 VID/PID 自动标成 K99 或 G75，必须先通过型号/协议应答区分。 |
| MCHOSE G75 Pro | `3554:FA09`，MCHOSE 20 字节无线键盘协议 | [`mchose-g75-pro`](COMPATIBILITY.md) | Peripheral Desk 有独立设备适配；其验证记录与 Voltide 上游贡献者样本分别记录。 |
| ROG Azoth Extreme | `0B05:1ACE` Omni 接收器，64 字节帧 | 与 [`asus-rog-scope-ii-96-omni`](COMPATIBILITY.md) 共用接收器标识 | 接收器响应不能可靠区分其当前配对的具体键盘；不额外创建会造成重复识别的插件。 |
| Razer Viper Ultimate、Viper V3 Pro、DeathAdder V3/V4 Pro、Basilisk V3 Pro 等无线型号 | Voltide 的 Razer provider 使用 91 字节 Feature Report；分别维护接收器 PID 清单 | 对照各 Razer 独立条目，见[兼容表](COMPATIBILITY.md) | 仅对目录中精确标注的具体 PID/连接复用。一个产品系列或同一报文长度不意味着任意 Razer HID 都可读取。 |
| Razer BlackShark V2 Pro | `1532:0555`，64 字节音频接收器通道 | [`razer-blackshark-v2-pro-2023`](COMPATIBILITY.md) | 已有 2023 版专用插件；不推广至其他 BlackShark 代际。 |
| ATK A9 Air 大师+ | 接收器 `373B:1278`，`FF02`，17 字节输入/输出；`HidD_SetOutputReport` 查询 | `atk-a9-air-master-plus` | 本批新增独立只读电量插件；与 A9 Plus Nearlink (`1115`/`10C9`) 严格分开。Voltide/贡献者报告为真机验证；本项目未实测。 |
| ATK N9 V2 Ultimate | 接收器 `373B:1262`，`FF90`，64 字节输入/输出；`WriteFile` 查询 | `atk-n9-v2-ultimate` | 本批新增独立只读电量/已知充电状态插件；不与同 VID/PID 的 ATK 鼠标接口混淆，限定 `FF90` 集合。本项目未实测。 |

## 许可和实现来源

本批按 Voltide MIT 授权移植对应协议实现，保留上游版权与许可文本于 `THIRD_PARTY_LICENSES/voltide/`，并在两个插件包中附带该许可。来源事实和适配范围分别指向 Voltide `src/Providers/ATKA9AiraddProvider.cs` 与 `src/Providers/ATKN9V2UltimateProvider.cs`；插件仍以本仓库 ABI 1、Rust DLL、设备专属 key 和按需目录发布。Voltide 是电量监控程序，不能作为 DPI、回报率、改键、宏、灯效或音效的证据。

来源：

- [Voltide v2.1.2 发布页](https://github.com/sdparklab/Voltide/releases/tag/v2.1.2)
- [ATK A9 Air+ provider](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKA9AiraddProvider.cs)
- [ATK N9 V2 Ultimate provider](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/src/Providers/ATKN9V2UltimateProvider.cs)
- [Voltide MIT License](https://github.com/sdparklab/Voltide/blob/73f16fd2be44570c0a861cef6b4804d743656893/LICENSE)
