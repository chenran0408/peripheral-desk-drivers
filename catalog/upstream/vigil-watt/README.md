# 厂商研究资料

这里保留 Vigil Watt 采用“厂商、协议、型号”分层后发布在开发分支的三套完整数据，固定到 [6315988](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins)。本目录只随源码提供。可安装的外设盒插件见[厂商目录](../../README.md)。

| 厂商 | 型号与连接记录 | 可查资料 |
| --- | --- | --- |
| 迈从 | 63 条设备记录，另列 8 条传输通道 | [设备、共用接收器、回报率表](mchose-catalog.json) · [协议与接口](mchose.protocols.json) |
| 雷柏 | 195 条设备记录 | [设备、有线/接收器配对和验证能力](rapoo-catalog.json) · [协议与接口](rapoo.protocols.json) |
| 北通 | 110 条运行目录记录，另有 66 条官方 PID 登记 | [运行与官方目录、验证状态](betop-catalog.json) · [协议与接口](betop.protocols.json) |

记录数包含同一型号的不同路径。对照时同时使用厂商、型号、VID/PID、连接方式和 HID 接口。北通资料将官方登记、仅识别到身份、收到电量报告和实物验证分别记录；迈从通用接收器的 USB 身份、设备通过协议自报的身份各有独立字段。厂商目录里出现的设置范围仅用于研究，进入插件前需与该型号的事务和证据相符。

文件沿用上游原文和 MIT 许可证。部分 `.json` 使用上游允许的注释，直接阅读时请使用支持 JSONC 的工具。更新时先核对固定提交的 `src/VigilWatt.Core/Plugins`，按[来源清单](sources.json)同步相应文件和[许可证](LICENSE)，在 PR 里注明改动影响哪个外设盒插件，并保留缺少实测的连接方式。
