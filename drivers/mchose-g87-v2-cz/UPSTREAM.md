# 迈从 G87 V2 · CZ 接收器

Archer-Wen 在 G87 V2 上核对了原厂 M HUB 网页、电量下降、插拔充电线和充满时的状态。固定参考为 Vigil Watt 的开发分支提交 `6315988f54e47fc030ae4c535dc64ae27c08f471`。本插件根据其公开记录独立编写 Rust 实现。外设盒尚未用该接收器实测。

在驱动商店选择预览版，搜索 **G87 V2**，选 **CZ 接收器** 后确认自己的键盘型号。插件只对 `3837:3033` 的 `MI_02 / FF31:0074 / 65 字节输入输出` 接口查询第一块状态，显示整数百分比与充电状态。它不会改变键盘设置。此接收器也用于 G75 V2、K87S 和 K99 V3，这三个型号的电量协议仍待验证。原有 G87 V2 的 `3837:201B` 继续使用独立的 V1 会话插件。

- [贡献者实物记录及原厂读数对照](https://github.com/evren-oct/vigil-watt/pull/104)
- [固定协议规格](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/specs/mchose.protocols.json)
- [固定设备表](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/mchose-catalog.json)
- [上游样本与充电状态判断](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/tests/VigilWatt.Core.Tests/MchoseCzSpecTests.cs)
