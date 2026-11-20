# 北通宙斯 2 / T9 接收器

本插件在 `20BC:5073` 接收器的 Col04 接口监听 24 字节、报告 ID 为 04 的状态。使用报告第 1 字节（从 0 开始数）的低三位读取档位：0、1、2、3、4 分别对应 0%、50%、80%、100%、100%。其他档位以未知显示。第 23 字节的占位值用于识别连接状态。该帧未提供充电位。

Vigil Watt 在 `6315988f54e47fc030ae4c535dc64ae27c08f471` 的记录中整理了维护者实物的 248 条 T9 样本，并根据北通客户端的位域修正早期估算。上游未逐次对照官方界面的百分比。外设盒独立编写 Rust 实现，依据已记录的档位检查解析，本项目未实机复测。

- [固定协议规格及档位表](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Betop/specs/betop.protocols.json)
- [T9 的上游硬件记录与修正](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/betop.md)
- [上游设备表中宙斯 2 / T9 对应关系](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Betop/betop-catalog.json)
