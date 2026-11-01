# 兼容与验证记录

以下包均为 ABI **1**、Windows **x64**，主程序 **>=0.4.0、<0.5.0**（MagDock 灯效编辑器需 **>=0.4.8**；本次雷蛇灯效编辑器需 **>=0.4.9**）。四款原有设备包为 **stable**；G304、G305、Basilisk V3 Pro 有线、A7 V3 Ultra+ 和 VGN F2 Master+ 仅在 **preview**，公开实现适配未实测。

设备包独立版本：四款稳定包为0.4.2；G304为0.4.13-preview.14；G305为0.4.1-preview.1；Basilisk V3 Pro有线为0.4.3-preview.1。它们继续使用ABI 1及上述0.4.x主程序范围，包版本变化不等于新增实机验证。

| 设备 | 已有核心适配实测连接 | 固件 | 初始包能力 | 边界 |
| --- | --- | --- | --- | --- |
| 炼狱蝰蛇 V3 专业版 | 2.4 GHz 接收器 1532:00B7 | 未知 | 电量、设置；标记0槽改键与软件宏触发 | 本机前进侧键已完成写入/恢复；标记1槽与左键保持只读，USB连接未测 |
| A5 Ultra 初代 | 2.4 GHz 接收器 3837:100C | 未知 | 电量、设置、六键改键；宏为实验功能 | 本次5个非左键均返回长度0宏页，未能确认可完整恢复，已安全拒写；仅安全页存在时才开放最多13事件的单次宏 |
| G75 Pro | 2.4 GHz 接收器 3554:FA09 | 未知 | 电量、设置、普通/Fn改键、灯色 | 宏地址未确证，未开放设备宏写入 |
| G2 Pro | USB 35BB:A100 | 未知 | 音量、原有音效、灯光 | 音效需官方组件；灯光开关已验证切换与恢复，模式/RGB未在本次写入验证，只显示本工具保存值 |
| Logitech G305 LIGHTSPEED（preview） | 未实测；仅接收器 046D:C53F、配对槽 1，须同时确认 WPID 4074、HID++ 4.2 与 G305 精确名称 | 本机未知；上游公开版本不算本机验证 | 仅公开只读电量与身份查询，无设置写入 | 不支持 G304、其他接收器/多槽或蓝牙；尚未实测 Windows 电量、唤醒、拔插与并发，详见 [G305 范围](LOGITECH-G305.md) |
| Logitech G304 LIGHTSPEED（preview） | 未实测；仅接收器 046D:C53F、配对槽 1，须同时确认 WPID 4074、HID++ 4.2 与 G304 精确名称 | 本机未知；上游公开版本不算本机验证 | 仅公开只读电量与身份查询，无设置写入 | 不支持 G305、其他接收器/多槽或蓝牙；尚未实测 Windows 电量、唤醒、拔插与并发，详见 [G304 范围](../drivers/logitech-g304/UPSTREAM.md) |

此表区分已有核心协议验证和新 DLL 的 ABI/打包检查；不把软件检查写成实机通过。连接模式或固件尚未单独确认时应明确保留未知。

Basilisk V3 Pro 有线预览仅匹配 `1532:00AA`、唯一 `MI_00` / 91 字节 Feature 集合，只读电量与充电状态。Windows 报告长度依据公开协议推导，尚无该型号本机验证；不包含 `00AB` 接收器、35K 变种、蓝牙或设置写入。详见[固定协议依据](RAZER-BASILISK-V3-PRO.md)。

A7 V3 Ultra+ 与 VGN F2 Master+ 包版本为 **0.4.4-preview.2**。A7 仅 `3837:1018` / `FF01:0001` / 64 字节输入输出，产品名必须精确匹配；F2 Master+ 仅 `3554:FB56/FB57` / `FF02` / 17 字节输入输出。两者只读电量与充电状态，尚未实机验证，不开放设置、改键、宏或灯光。固件兼容范围未知。详见 [A7 范围](MCHOSE-A7-V3.md)与 [F2 Master+ 范围](VGN-F2-MASTER-PLUS.md)。

0.4.5-preview.1 新增 **ATK X1 Pro Max**（有线 373B:1017 / 8K 接收器 373B:101B，CID/MID 必须 2,39）及 **Viper V2 Pro 有线**。两款均为公开实现适配、尚未实机验证；仅只读电量/充电状态，不提供设置写入。详见 [ATK 范围](ATK-X1-PRO-MAX.md)与 [Viper 范围](RAZER-VIPER-V2-PRO.md)。

0.4.6-preview.1 继续新增 **ATK A9 Plus Nearlink**（有线 373B:1115 / NearLink 接收器 373B:10C9，CID/MID 2,83）及 **Viper Ultimate 有线**（1532:007A）。只读电量/充电状态，未实机验证，不开放写入。详见 [A9 范围](ATK-A9-PLUS-NEARLINK.md)与 [Viper Ultimate 范围](RAZER-VIPER-ULTIMATE.md)。

0.4.7-preview.1 新增 **VGN F2 Pro Max**、**VGN F1 Pro** 的有线/接收器电量与充电状态，以及 **DeathAdder V2 Pro 有线**。均依据上游已有实现接入，移植后尚未实机复测，暂不提供设置写入；精确来源和范围见 [F2 Pro Max](VGN-F2-PRO-MAX.md)、[F1 Pro](VGN-F1-PRO.md)、[DeathAdder V2 Pro](RAZER-DEATHADDER-V2-PRO.md)。

0.4.10-preview.1 新增 **迈从 V9 Pro** 接收器电量读取。仅支持 `291D:385D`、`MI_00/COL05`、`FF90:0001` 64 字节电量接口；安装后主动查询。上游为贡献者单台设备的 100% 抓包，本项目未实测；充电状态和非满电映射未知。详情见 [V9 Pro 支持范围](MCHOSE-V9-PRO.md)。

## 0.4.8-preview.1 设置与灯效扩展

| 型号 | 连接 | 功能 | 验证范围 |
| --- | --- | --- | --- |
| AJAZZ AJ179 PRO | USB 3151:4026 / 接收器 3151:402D，运行时型号 1851 | DPI、回报率、休眠、鼠标键/DPI循环映射 | 上游 v3.03 实机记录；本项目未复测，蓝牙不开放 |
| Viper V2 Pro | 有线 1532:00A5 | 电量、DPI、125/500/1000 Hz | 固定 OpenRazer 实现；本项目未实测，无新增改键或灯效 |
| MCHOSE MagDock | 独立底座 3837:1012 | 灯光开关、六模式、适用模式 RGB、速度/亮度 | 上游读写恢复记录；本项目未复测；需主程序 0.4.8 |

详见 [AJ179 PRO](AJAZZ-AJ179-PRO.md)、[Viper V2 Pro](RAZER-VIPER-V2-PRO.md)、[MagDock](MCHOSE-MAGDOCK.md)。旧版 Viper 只读说明由本节新增能力取代；其余旧包本次不变。

## 通道

- stable：可在普通驱动商店选择。兼容范围明确，说明仍保留已知限制。
- preview：需用户主动切换到预览通道；不能自动覆盖稳定包。预览目录可能暂时为空。

构建会生成 catalog.json（stable 别名）、catalog-stable.json 与 catalog-preview.json。每个条目自带 channel、core_min、core_max_exclusive、capabilities、用户向 changelog。ABI 不兼容必须升级 ABI 与主程序，不能仅靠版本号绕过。

## 0.4.9-preview.1 当前扩展范围

本节取代上文对应型号的旧版只读说明；历史记录不代表当前包仍只有电量功能。八款更新包均依据固定上游版本适配，**尚未在本项目实机验证**；型号与连接方式保持各型号文档的精确限制。

| 型号 | 新增功能 | 限制 |
| --- | --- | --- |
| A7 V3 Ultra+ | DPI、回报率、鼠标键映射 | 无灯效；未提供设备宏 |
| VGN F2 Master+ | DPI、当前档位切换、回报率、鼠标键映射 | 改键沿用上游共享布局，缺少该型号厂商抓包；无宏或灯效 |
| ATK X1 Pro Max | DPI、回报率 | 不含 1K 接收器；无改键或灯效依据 |
| ATK A9 Plus Nearlink | DPI、回报率 | 最高 1000 Hz；无改键或灯效依据 |
| Basilisk V3 Pro 有线 | DPI、回报率、常亮/光谱/波浪、亮度和开关 | 仅 1532:00AA，不含接收器与 35K 版 |
| Viper Ultimate、DeathAdder V2 Pro 有线 | DPI、回报率、Logo 灯效、亮度和开关 | 精确模式见型号文档，未开放接收器设置 |
| Logitech G305 | DPI 与设备报告的 DPI 选项 | 仅精确型号；板载配置开启时不开放写入，不自动改动板载模式 |

雷蛇灯效的亮度可读回，颜色与模式只能显示本工具上次发送的设置。首次显示的颜色是待应用草稿，不是从设备读出的状态。关键设置写入失败会尝试恢复原值，并报告恢复结果；未知灯效无法完整恢复时会明确说明。

这批功能由设备固件处理，不需要软件宏常驻。F1 Pro、F2 Pro Max 本次保留电量支持，不把缺乏型号依据的设置伪装为已支持。安装本批插件前建议先升级主程序至外设盒 0.4.9。

0.4.9-preview.3 补齐 G305、F2 Master+ 的主界面 DPI 范围声明和三款雷蛇的灯效入口声明；协议范围不变。

## G2 Pro 0.4.10 完整组件包

G2 Pro 包需主程序 >=0.4.10，内含匹配 35BB:A100 的 x64 SDK、音频驱动、扩展和 APO，完整清单与来源见 [组件说明](G2-PRO-COMPONENTS.md)。插件会自动连接 SDK；系统驱动由用户在 EQ 页点击安装并通过 Windows 管理员与签名检查。旧 11.1.1.6 驱动的音效读取已实测；附带 11.2.1.0 / APO 2.1.0.2 的系统安装本轮未在用户机器执行。插件回退不回退 Windows 驱动。

## ATK A9 Air 大师+（preview）

仅支持 `373B:1278` / `FF02:0002` / 17 字节接口的 2.4 GHz 接收器，读取电量、充电标志和电压。协议依据 Voltide v2.1.2 贡献者对该精确型号/接收器的真机验证记录；Peripheral Desk 尚未独立实测。不覆盖 NearLink、有线、蓝牙或其他 ATK 型号，不提供 DPI、回报率、改键、宏或灯效。详见 [ATK A9 Air 范围与来源](ATK-A9-AIR-MASTER-PLUS.md)。

## ATK N9 V2 Ultimate（preview）

仅支持 `373B:1262` 接收器的 `FF90`、64 字节接口，通过中断输出查询，读取电量和已知充电状态。协议依据 Voltide v2.1.2 对该精确型号/接收器的贡献者真机验证记录；Peripheral Desk 尚未独立实测。不提供 EQ、虚拟 7.1、灯效或其他控制功能。详见 [ATK N9 V2 Ultimate 范围与来源](ATK-N9-V2-ULTIMATE.md)。

## Logitech Voltide 设备（preview）

Voltide v2.1.2 已报告 G304（`046D:C53F`，与 G305 共用接收器 PID）、G Pro X Superlight 2（GPW3，LIGHTSPEED `046D:C547/C54D` 与有线 `C09B`）、M750M（`046D:C548`）和 G102（`046D:C539`）的 HID++ 电量读取。Peripheral Desk 为每个精确型号提供独立只读插件，并校验 HID++ 4.2 ping；G304/G305 还必须匹配 WPID `4074` 和完整设备名，避免共用接收器误认。本项目尚未独立实测。不开放 DPI、回报率、改键、宏或灯效。

## Voltide 共享身份缺口

| Voltide 型号 | 共享接口 | Peripheral Desk 决策 | 兼容边界 |
| --- | --- | --- | --- |
| MCHOSE K99 v2 | `3554:FA09`、`FF02`、Report ID `0x13`、20 字节输入/输出 | [`mchose-k99-v2`](../drivers/mchose-k99-v2) preview，共用接收器，手动认领 | 仅提供只读电量及已知充电状态。Voltide v2.1.2 的上游验证范围是共用接收器的 K99 v2 电量；本项目未实机复测。接收器无法证明具体键盘型号，用户须先确认；无设置、改键、宏和灯光。 |
| MCHOSE G75 Pro | 同上 | [`mchose-g75-pro`](../drivers/mchose-g75-pro) stable | 电量、设置、普通/Fn 改键与灯色沿用现有 G75 协议；Voltide 的 36% 贡献者样本不等同于本项目重新实测。其他 `3554:FA09` 键盘不因 PID 自动获得 G75 能力。 |
| ROG Azoth Extreme | `0B05:1ACE` Omni 接收器、`FF00:0001`、64 字节输入/输出 | [`asus-rog-azoth-extreme`](../drivers/asus-rog-azoth-extreme) preview，共用接收器，手动认领 | 仅提供只读电量与充电状态。Voltide v2.1.2 上游验证范围是 Azoth Extreme 的 Omni 状态帧；本项目未实机复测。配对回复不能证明具体型号，用户须先确认；无设置、改键、宏和灯光。 |

K99 V2 和 Azoth Extreme 的手动认领只依赖用户确认型号，不能视作运行时型号识别；两个包均不会因共享接收器自动认领。G75 Pro 的独立型号签名与 Scope II 96 的配对产品 ID 校验仍分别约束原有插件。


## Voltide v2.1.2 全清单补齐

新增 preview 插件均为只读电量/已知充电信息，逐设备按 Voltide 上游标出的 PID、接收器和接口匹配；上游有实物验证报告，本项目没有逐台实测。罗技 G304 与 G305 共用 `046D:C53F`，插件还校验 WPID `4074` 与完整型号名。Razer 使用 `1532`、91 字节 Feature、`MI_00` 接收器路径，并按对应 PID 单独安装；原始电量字段保留为诊断值，百分比遵循 Voltide 的 0–100 原值或 0–255 归一化换算。

| 新增型号 | 连接标识 | 插件 | 能力 |
| --- | --- | --- | --- |
| Logitech G304 | `046D:C53F`，HID++ 4.2，WPID `4074`，G304 设备名 | `logitech-g304` | 电量、充电状态 |
| Razer Viper Ultimate 无线 | `1532:007B` | `razer-viper-ultimate-receiver` | 电量、充电状态 |
| Razer Viper V3 Pro | `1532:00C1` | `razer-viper-v3-pro` | 电量、充电状态 |
| Razer DeathAdder V3 Pro（备用接收器） | `1532:00C3` | `razer-deathadder-v3-pro-alt` | 电量、充电状态 |
| Razer DeathAdder V4 Pro | `1532:00BF` | `razer-deathadder-v4-pro` | 电量、充电状态 |
| Razer Basilisk V3 Pro | `1532:00AB` | `razer-basilisk-v3-pro` | 电量、充电状态 |
| Razer Basilisk V3 Pro 35K | `1532:00CD` | `razer-basilisk-v3-pro-35k` | 电量、充电状态 |
| Razer Basilisk V3 Pro 35K Phantom | `1532:00D7` | `razer-basilisk-v3-pro-35k-phantom` | 电量、充电状态 |
| Razer DeathAdder V3 HyperSpeed | `1532:00C5` | `razer-deathadder-v3-hyperspeed` | 电量、充电状态 |
| Razer Pro Click V2 | `1532:00D1` | `razer-pro-click-v2` | 电量、充电状态 |
| Razer Pro Click V2 Vertical | `1532:00C8` | `razer-pro-click-v2-vertical` | 电量、充电状态 |
| Razer Basilisk Mobile | `1532:00D4` | `razer-basilisk-mobile` | 电量、充电状态 |
| Razer Cobra Pro / Cobra HyperSpeed | `1532:00B0` / `00DB` | 对应两个独立插件 | 电量、充电状态 |
| Razer Naga V2 HyperSpeed | `1532:00B4` | `razer-naga-v2-hyperspeed` | 电量、充电状态 |
| Razer Viper V3 HyperSpeed | `1532:00B8` | `razer-viper-v3-hyperspeed` | 电量、充电状态 |
| Razer Basilisk V3 X HyperSpeed | `1532:00B9` | `razer-basilisk-v3-x-hyperspeed` | 电量、充电状态 |
| Razer Pro Click Mini | `1532:009A` | `razer-pro-click-mini` | 电量、充电状态 |
| Razer Lancehead | `1532:005A` | `razer-lancehead` | 电量、充电状态 |
| Razer Mamba Chroma | `1532:0045` | `razer-mamba-chroma` | 电量、充电状态 |

G304/G305 身份会在接收器应答后解歧；仅相似 VID/PID 不够。MCHOSE K99 v2 与 G75 Pro 共用 `3554:FA09`，ROG Azoth Extreme 与 Scope II Omni 共用 `0B05:1ACE`；K99 v2 和 Azoth Extreme 只提供手动认领的预览包，不通过接收器自动确认型号。

## 六款上游实机记录设备（preview）

这六个独立插件只读取电量及协议中可知的充电状态；上游项目有实物验证记录，Peripheral Desk 尚未独立实测。NZXT CAM 同时运行时 Lift Elite 接收器可能不回应；LAMZU Maya X 的有线 ID 上游没有单独确认；ROG Gladius III Wireless 不支持蓝牙；Corsair Dark Core RGB Pro SE 只收录 `1B1C:1B7F` 接收器，回复没有充电字段。各型号连接、协议及来源见[设备目录](DEVICE-CATALOG.md)和相应驱动包内的 UPSTREAM/PROVENANCE 文件。HaloBattery/rivalcfg 许可文本位于仓库 `THIRD_PARTY_LICENSES`；rogdrv-web 的 GPL-3.0 实现只用于参考协议事实，未复制代码。
