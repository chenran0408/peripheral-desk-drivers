# 迈从通用 8K 接收器读取范围

在预览商店按完整型号安装插件，即确认允许外设盒查询该接收器。日常刷新通过 `0x0900` 读取配对鼠标的内部 VID/PID、电量、充电标志和连接状态。运行时同时核对接收器 `3837:1018`、HID `FF01:0001`、64 字节输入和输出，以及回复中该款鼠标的配对身份。休眠短应答、0% 占位或配对其他型号时，不提供电量。

打开支持指针参数读取的设备概览或点击“读取指针参数”时，外设盒发送额外的只读查询。普通电量刷新只顺带接收等待窗口里的档位事件。插件只在用户确认的型号应答后读取配置，一次详情请求最多额外发三条。

| 读数 | 取值方式和含义 |
| --- | --- |
| 连接与充电 | 连接值 1 表示有线，2 表示 2.4 GHz，其余未知。充电值 0 表示未充，1 表示正在充，2 表示已满。未知充电值保持空。 |
| 配置回报率 | `0x0002` 返回有线和无线两个档位索引。界面只在二者一致、官方型号加接收器的表中存在该档位时显示换算所得的 Hz。两个索引均随有效应答返回。没有实测吞吐率。 |
| X/Y 轴 DPI 配置 | `0x0003` 读取配置索引 0。回复须回显所请求的配置及方向，提供六个小端槽位，并声明 1 到 6 个有效档位。X 应答声明支持独立 Y 轴时再读 Y。各轴只显示已启用且数值有效的档位。该配置是否正在使用尚无取证，本插件不据此生成“当前 DPI”。 |
| 当次上报的活动档位 | 收到 `0x0600` 的档位事件且索引为 0 到 5 时，返回该索引。设备详情按第 1 到第 6 档显示。 |

率或轴配置读取失败时，已经取得的电量、另一项配置和可解析的索引仍可用。此批只支持上述接收器和读取命令。其他无线接收器、直连路径和修改设置需要独立验证。

## 依据与验证

Vigil Watt 的物理样本在该接收器上自报 `3837:4026`、100% 电量、充电标志 0 和连接方式 2。上游现场记作 K7 V2 Ultra+，官方表将此配对编号分配给 A5 V3 Ultra+，与另一个本仓库既有 A7 V3 Ultra+ 样本也有冲突。本批绕开 `4026`。K7 V2 Ultra+ 插件只接受其官方编号 `4028`。逐款对照 16 个鼠标内部 PID 以及官方配置里的 `1018` 接收器槽位后，范围包括 A5 V3 三款、A7 V3 六款、K5 两款、K7 V2 四款及 R7 Ultra。原有 A7 V3 Ultra+ 插件继续按其独立取证条件工作。

上游已经解析 DPI 配置及换算回报率，并给出合成应答测试。上述 16 个型号的这些功能尚无逐款物理验证。本项目使用上游接收器抓包和合成应答测试解析、配对隔离和配置规则，尚未进行实机测试。

- [接收器抓包、命令及现场记录](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/mchose.md)
- [厂商前端和型号/接收器的交叉对照](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/reference/mchose-dpi-report-rate-audit.md)
- [官方表整理版](../../catalog/upstream/vigil-watt/mchose-catalog.json)
- [上游状态和指针参数查询](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/MchosePlugin.cs)
- [上游编解码](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/Mchose0x4DCodec.cs)
- [回报率换算条件](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/MchoseRatePolicy.cs)

接收器适配以 Rust 独立实现。上游数据与抓包的 MIT 许可随每个包提供。[许可证](VIGIL-WATT-MIT.txt)
