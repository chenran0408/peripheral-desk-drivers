# Razer / 雷蛇

现有包：`razer-deathadder-v3-pro`，原已接入1532:00B6有线 / 00B7接收器。实机与恢复状态以包metadata及既有记录为准；不增加未测场景的硬件认证声明。

新增预览包：`razer-basilisk-v3-pro-wired`，仅USB有线 `1532:00AA`，只读电量/充电状态，公开实现适配未实测。它只打开唯一匹配的 `MI_00`、91字节Feature接口；不支持00AB接收器、35K变体、设置写入、按键、宏或灯光。协议和固定上游来源见 [Basilisk V3 Pro 有线说明](../RAZER-BASILISK-V3-PRO.md)。

OpenRazer有明确DeathAdder V3 Pro设备类及电量、充电、DPI档位、回报率、休眠和阈值方法；还列出00C2/00C3另一组ID，不能据此把本项目现有00B7实测自动推广。OpenMouse Razer改键codec的公开验证型号与本机型号须区分；本机现有改键依赖本项目自己的实际回读验证。HyperPolling共享接收器还需先确认配对鼠标身份。

后续可选Viper V2/V3 Pro精确ID的只读电量，但先核对型号、report结构/长度、充电查询和固件差异。板载宏未获型号级依据时不开放。

完整来源与许可证见 [调查矩阵](../PROTOCOL-RESEARCH.md)。
