# MCHOSE MagDock 灯效

此预览驱动支持 MCHOSE MagDock 充电底座 `3837:1012` 的灯效读取、开关、模式、RGB、速度和亮度设置。底座不是鼠标本体；它使用独立 HID 集合 `FF00:0001`。当前只匹配 Windows HID 枚举到的唯一、64 字节输入与输出集合。

灯效编号沿用 MCHOSE 枚举：`0` Static、`1` Breathing、`2` Shining、`3` Cycling、`4` Go Flow、`5` Music。速度与亮度范围均为 `0–4`。RGB 只对 Static 与 Breathing 有效果。关闭灯效只将 enabled 设为 false，保留模式。Music 模式须同时开启 music_sync。

写入命令携带完整的十个灯效参数，没有单字段写入。驱动先重新读取设备，保留 effect count、direction 和未编辑的值，再写入；写后读取并核对全部可写字段。核对或读取失败时尝试写回并核对原始字段。读回响应中其他保留/回显字节会留在协议层，但协议没有定义它们为写入参数，因此不会伪造它们的写入语义。未知 effect 编号、越界速度或亮度均拒绝写入。

## 验证与来源

上游 OpenMouse Project `mouse-protocol` 固定提交 [`bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc) 对真实 MagDock 完成灯效读取、原状态无改动回写、亮度及静态颜色修改、恢复，并记录每步读回一致及最终整块字节一致。依据文件包括 [`src/mchose/dock.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/dock.ts)、[`src/drivers/mchose/dock-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/dock-hid.ts)、[`src/drivers/mchose/dock-hid.test.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/dock-hid.test.ts) 与 [`docs/mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md)。上游仓库为 AGPL-3.0-or-later。本插件独立移植协议，不复制上游源码；协议事实和证据按原项目归属。

本项目没有连接 MagDock 实测。Windows 报告长度映射（未编号的上游 63 字节 WebHID 报告加 Windows HID 的 report ID 字节 0）已在移植中明确处理，实际设备描述符及安装升级数据保留仍需本项目环境验证。图片为中性占位图，不是 MagDock 实物照片。

本包 0.4.8-preview.2 要求主程序至少 0.4.8，修复旧主程序拒绝 accessory 设备类型的目录兼容问题；协议功能与 preview.1 相同。
