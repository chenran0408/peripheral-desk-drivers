# 迈从 G3 V2

安装后可在外设盒主页和托盘看到鼠标电量和充电状态。适用于 `3837:4245` 的 2.4 GHz 接收器。休眠或本次查询无回复时，以主程序标明的新鲜度为准。

贡献者记录了接收器在 M HUB 查询后返回 89% 且未充电。该接口是 `FF01:0010`，输入和输出报告长度均为 65 字节。外设盒通过同一命令获取 1–100 的整数电量和充电位，支持 Windows 缓冲区带额外报告 ID 的格式。来源中的 0 为 “尚未计算” 占位。外设盒已用上游样本检查解析，本项目尚未用实机验证。固件版本未记录。

协议依据 Vigil Watt 开发提交 `6315988f54e47fc030ae4c535dc64ae27c08f471` 和上游引用的迈从 M HUB Gmouse。上游 MIT 许可随包提供。Rust 适配复用本项目 G7 已有的 Gmouse 查询与 HID 传输，按 G3 V2 独立核对接收器和用法。

[贡献者实物与命令记录](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/mchose.md) · [上游协议规格](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/specs/mchose.protocols.json) · [实物回复样本](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/tests/VigilWatt.Core.Tests/MchoseGmouseCodecTests.cs) · [上游 MIT 许可](UPSTREAM-MIT.txt)
