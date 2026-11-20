# 迈从 K5 V2 · 3837:101C 接收器

插件使用 `FF01:0001` 的 64 字节 0x4D 通道，执行每次读写前查询 `0x0900`，设备须自报 `3837:4040`。此包覆盖 101C 接收器。原厂表还列有 4040 有线模式，此连接未在该包声明。

在外设盒中可查看电量、充电和 DPI 档位，设置最高 42,000 DPI、125/500/1000/2000/4000/8000 Hz 和 0.7/1/2 mm 抬升距离。五键示意图可用于改键与设置 DPI 加减档，板载宏根据设备回复的容量开放。原厂 `newMouse3395` 列出 K5 V2，原生编辑器按该分支隐藏玻璃模式。

原厂 `newMouseWirelessPids`、`mouseDiffConfig` 对应型号、接收器、设置上限及回报率表；`supportDpiKeyDevice` 不含 K5 V2。Vigil Watt 的固定目录记录该接收器为 0x4D 通道。此前上游实物样本来自另一只 1018 接收器，本项目用合成样本核对这款鼠标的型号边界与取值范围，尚无 K5 V2 实机读写记录。[原厂前端](https://www.mchose.com.cn/assets/main-DIt5vAcV.js) · [原厂 K5 系列说明书](https://cdn.mchose.com.cn/configCenter/static/documents/K5_9tso4iayd4sxc6dx.pdf) · [上游固定型号目录](../../catalog/upstream/vigil-watt/mchose-catalog.json) · [协议对应表](../../crates/protocol-mchose-0x4d/UPSTREAM.md)
