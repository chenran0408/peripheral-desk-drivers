# 五品牌协议调查与 11 个参考项目核查

核查日期：2026-10-21。下列固定版本的源码/设备定义已实际查看；“上游实现”“本项目实现”“本机实测”分别记录。共享 VID、共享接收器、列出型号或 GUI 可打开都不足以宣称可用。

后续执行见[接入任务队列](ADAPTATION-ROADMAP.md)。新增 [Basilisk V3 Pro 有线只读预览](RAZER-BASILISK-V3-PRO.md)使用本仓库独立 Razer 协议模块，未把新型号实现放回主程序；仍标注未实机验证。下表保留初次调查范围，新增型号的精确依据见对应文档。

## 本仓库品牌目录

| 品牌 | 已落地范围 | 下一步与边界 |
| --- | --- | --- |
| [MCHOSE](brands/MCHOSE.md) | A5 Ultra 初代、G75 Pro 的独立包，以及 V9 Pro 接收器电量预览 | A5 初代与 V2/A7 的性能字段不同，不能按同品牌推广；V9 仅有上游单点样本；原厂宏不能完整备份时拒写 |
| [ATK](brands/ATK.md) | 本轮调查候选，尚无 ATK 可安装支持声明 | 先选精确接收器/鼠标 CID/MID 与公开只读事务，取得实物后再提高验证等级 |
| [VGN](brands/VGN.md) | 本轮调查候选，尚无 VGN 可安装支持声明 | F1、F2、F2 Pro Max 的 PID/协议不能互相替代；与 MCHOSE 共用 3554 不代表兼容 |
| [Logitech](brands/LOGITECH.md) | G305 专用只读 preview 插件已实现、未实测 | C53F + WPID4074 + G305 精确名 + HID++4.2；未开放任何设置写入 |
| [Razer](brands/RAZER.md) | 已有 DeathAdder V3 Pro 独立包 | 00B6/00B7 与其他版本/HyperPolling 分开；其他鼠标的按键 codec 不直接视为本型号证据 |

## 逐项核查：10 组、11 项

| 项目 / 固定版本 | 实际检查位置与证据 | 能借鉴的范围 / 不能据此声称的内容 |
| --- | --- | --- |
| [OpenRazer 477f4d6](https://github.com/openrazer/openrazer/tree/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d) | `driver/razermouse_driver.{c,h}`、`razerchromacommon.c`、`daemon/openrazer_daemon/hardware/mouse.py` 的 DeathAdderV3Pro 类：1532:00B6/00B7，另列 00C2/00C3；METHODS 明确电量、充电、DPI/档位、回报率、休眠、低电阈值 | Razer 电源与设置字段依据；该型号方法列表没有板载宏/通用改键承诺。Linux 内核模块+daemon 不能直接成为 Windows 运行依赖。代码 GPL-2.0-or-later，文档另有 CC-BY-SA |
| [libratbag 8235b5b](https://github.com/libratbag/libratbag/tree/8235b5bb6032dea15901ab58a8218956f4494d08) | `data/devices/logitech-g305.device` 为 usb:046d:4074 / hidpp20 / G305 quirk；完整设备定义目录按品牌/内容检索未找到 MCHOSE、ATK、VGN 对应定义 | HID++ 和逐型号 quirk 可参考；4074 是鼠标身份，不能当 C53F receiver 的 USB PID。设备表没有条目就不能因同品牌接入。MIT |
| [Piper 5d3c884](https://github.com/libratbag/piper/tree/5d3c8845b55643595ccb8029dfa5c7a2fb079e77) | README 明确它只是 ratbagd DBus GUI，设备支持取决于 libratbag；`piper/application.py` 调 ratbagd | 可借鉴能力驱动的 GUI 分工；它没有额外设备私有协议，不是第二套更广的支持库。GPL-2.0-or-later 源码头 |
| [Solaar e7304c4](https://github.com/pwr-Solaar/Solaar/tree/e7304c4c451cc9bb4f206a914844525e67856a28) | `base.py` 帧/软件ID/回复；`hidpp20.py` 的 get_ids/get_name/get_battery_status；G305 与 G304 的 4074 设备 dump，C53F receiver dump | 已有成熟 Logitech 公开特性电量；同一 WPID4074 仍需区分 G304/G305 名称。本轮 G305 的主要依据；上游记录不算本机实测。GPL-2.0-or-later |
| [OpenMouse mouse-protocol bac0315](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc) | `src/mchose`、`src/drivers/mchose/a5-gen1-hid.ts`；ATK products/device-catalog 的 CID/MID；`src/drivers/vgn/hid.ts` 限3554:FB56/FB57；Logitech HID++ 与 onboard-profile 文档；Razer button codec 的测试型号说明 | 覆盖多品牌但内部是多种 transport/codec。A5 初代与新版分开；ATK 公用 VID/PID 需 CID/MID 解歧；VGN F2 的 FB56/57 不等于其他 F2 后缀。Razer 某 codec 在 ViperV3Pro 验证，不自动覆盖 00B7；GPX 原版与 GPX2 profile 格式不同。AGPL-3.0-or-later |
| [OpenRGB 763e325](https://github.com/CalcProgrammer1/OpenRGB/tree/763e325f9b8afda33ed5dd9be73a0eee7be42bff) | `Controllers/LogitechController` 的 detector、HIDPP20/Windows transport；`Controllers/RazerController` detector 与 RGB controller；完整 Controllers 路径未发现以 MCHOSE、ATK、VGN 命名的控制器 | RGB 控制与 Windows 长报文选择可参考；不是鼠标全部驱动功能集合，不能从灯光支持推导 DPI、改键或宏，更不能从同接收器推导所有型号。所查看代码 GPL-2.0-or-later |
| [QMK 180f394](https://github.com/qmk/qmk_firmware/tree/180f394b489c238cb231c2bf8b080262155e03c0) | `docs/features/rawhid.md`：固件须编译 RAW_ENABLE，默认 FF60:61、固定32字节；主键盘目录未见以本轮五品牌明确命名的目标（目录名搜索不能排除 OEM/别名） | 是键盘固件，需该 PCB/MCU 可运行的固件与矩阵，不能装一个桌面插件就给原厂 G75 变成 QMK。不能据此刷未知固件。整体 GPLv2，部分子模块/文件不同，复用须逐文件核对 |
| [VIA app 935106a](https://github.com/the-via/app/tree/935106a990af39bdd1ade881c02366eac486240d) | README 的准入条件：QMK 固件、VIA keymap、keyboard definition；`src/utils/hid-keyboards.ts`/`keyboard-api.ts` 按定义和协议连接 | 实时改键 GUI/JSON 布局可参考；目标键盘须已有 VIA 固件和正确 JSON。未找到 G75 Pro 支持证据，不能把 VIA JSON 当原厂协议转换器。app GPL-3.0 |
| [Vial GUI aef8222](https://github.com/vial-kb/vial-gui/tree/aef8222a2d0429a183b2ed692d5f9efcfd383f08) | `src/main/python/protocol/keyboard_comm.py` 检查 VIA/Vial protocol；`constants.py` 的 FE prefix、UID/definition/unlock/宏命令 | Vial 固件的发现/布局/宏/解锁模型可参考；须 Vial-enabled keyboard，不能沿用 FE 命令给原厂 G75/其他 HID。所查看代码 GPL-2.0-or-later |
| [quiet-battery d6eb4bd](https://github.com/TheWebDog/quiet-battery/tree/d6eb4bdade47dccd7cb402e16ac50a6f351fcac0) | README、`docs/DEVICE-SUPPORT.md`、`docs/VALIDATION.md`：G304 C53F 为实测；Logitech其他目标、ATK/VXE/VGN精确ID表、Razer新增型号明确“已实现待实测” | 电量产品流程与范围界限有参考意义。它区分 G304 实测与家族未实测，不能把后者转述成全部真机支持。**PolyForm Noncommercial 1.0.0，非通常开源许可；本项目不复制其源码/图像或将其作为 AGPL 代码依赖。** |
| [2.4G-battery-viewer f0e983e](https://github.com/yszyszysz233/2.4G-battery-viewer/tree/f0e983e88900592237c7df9ae6b80b9dbeb3020b) | README 明示仅两台：373B:1216 ATK/VXE、3654:6901 DARKRIM SHADOW；前者 FF05/report08 特定请求，后者 JL TLV/CRC；该版本根目录未见 LICENSE | 可参考 Windows HID 调查方法，不是 ATK 全系列电量表；373B:1216 特性报文与另一些17字节 Compx 事务不能混用。未发现明确授权则不复制源码，不把 DARKRIM 协议套到 G2 Pro |

表中“未找到”只指固定版本、所列检查范围。没有仓库条目不等于设备不可能适配；有同厂商号更不等于已适配。

## 接入次序

先维护现有设备功能和失败恢复；G305、VGN F1 Pro、迈从 V9 Pro 已有独立预览包。V9 Pro 依据 Vigil Watt v1.3.0（`a78e973`）协议资料独立实现，只支持接收器电量，且上游只有贡献者在 100% 的单点抓包、本项目未实测。后续新型号继续要求精确到连接与接口；来源只提供协议事实时不复制实现代码。

每款新插件须列：USB 接收器/有线精确ID、控制集合和报告长度、运行时型号/配对身份校验、来源固定 revision、协议命令只读性、固件和连接状态、测试层级、未覆盖能力。未知固件不标实测；有实机才把 validation.status 从 public-implementation-untested 提升为 hardware-verified。
