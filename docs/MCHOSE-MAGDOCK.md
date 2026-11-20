# MCHOSE MagDock 灯效

此预览驱动支持 MCHOSE MagDock 充电底座 `3837:1012` 的灯效读取、开关、模式、RGB、速度和亮度设置。底座不是鼠标本体；它使用独立 HID 集合 `FF00:0001`。当前只匹配 Windows HID 枚举到的唯一、64 字节输入与输出集合。

灯效有常亮、呼吸、闪耀、变色循环、流动和音乐六种。常亮、呼吸、闪耀和音乐可选基色。流动模式可设置顺时针或逆时针。速度和亮度均有五档。关灯后保留当前模式。音乐模式开启底座的同步开关，实际节拍取决于音频同步来源。

底座使用命令 7 读取当前方案。流动方向取自回复参数 29，写在命令 39 的第十个参数中。原始值 0 对应顺时针，1 对应逆时针。插件每次取回设备原值、写入并读回全部十个参数。当前模式允许调方向且用户选了方向时才修改该值。验证失败会尝试把原值写回并重新读取。

## 验证与来源

上游 OpenMouse Project `mouse-protocol` 固定提交 [`bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc) 对真实 MagDock 完成灯效读取、原状态无改动回写、亮度及静态颜色修改、恢复，并记录每步读回一致及最终整块字节一致。依据文件包括 [`src/mchose/dock.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/dock.ts)、[`src/drivers/mchose/dock-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/dock-hid.ts)、[`src/drivers/mchose/dock-hid.test.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/dock-hid.test.ts) 与 [`docs/mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md)。上游仓库为 AGPL-3.0-or-later。本插件独立移植协议，不复制上游源码；协议事实和证据按原项目归属。

本项目没有连接 MagDock 实测。Windows 报告长度映射（未编号的上游 63 字节 WebHID 报告加 Windows HID 的 report ID 字节 0）已在移植中明确处理，实际设备描述符及安装升级数据保留仍需本项目环境验证。图片为中性占位图，不是 MagDock 实物照片。

从 0.4.13-preview.45 起，插件提供流动方向以及闪耀、音乐的基色设置，依据 [M HUB 网页驱动](https://www.mchose.com.cn/) 及 [2026-11-20 的前端构建](https://www.mchose.com.cn/assets/main-DIt5vAcV.js) 的底座设置页面独立实现。固件编号未知，本项目以协议和编辑器交互检查覆盖新增项目。使用外设盒 0.4.11-preview.12 或更新版可选择方向；旧版允许继续使用其已有灯效入口。
