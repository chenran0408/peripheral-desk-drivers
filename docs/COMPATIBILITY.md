# 兼容与验证记录

以下包均为 ABI **1**、Windows **x64**，主程序 **>=0.4.0、<0.5.0**（MagDock 灯效编辑器需 **>=0.4.8**）。四款原有设备包为 **stable**；G305、Basilisk V3 Pro 有线、A7 V3 Ultra+ 和 VGN F2 Master+ 仅在 **preview**，公开实现适配未实测。

设备包独立版本：四款稳定包为0.4.2；G305为0.4.1-preview.1；Basilisk V3 Pro有线为0.4.3-preview.1。它们继续使用ABI 1及上述0.4.x主程序范围，包版本变化不等于新增实机验证。

| 设备 | 已有核心适配实测连接 | 固件 | 初始包能力 | 边界 |
| --- | --- | --- | --- | --- |
| 炼狱蝰蛇 V3 专业版 | 2.4 GHz 接收器 1532:00B7 | 未知 | 电量、设置；标记0槽改键与软件宏触发 | 本机前进侧键已完成写入/恢复；标记1槽与左键保持只读，USB连接未测 |
| A5 Ultra 初代 | 2.4 GHz 接收器 3837:100C | 未知 | 电量、设置、六键改键；宏为实验功能 | 本次5个非左键均返回长度0宏页，未能确认可完整恢复，已安全拒写；仅安全页存在时才开放最多13事件的单次宏 |
| G75 Pro | 2.4 GHz 接收器 3554:FA09 | 未知 | 电量、设置、普通/Fn改键、灯色 | 宏地址未确证，未开放设备宏写入 |
| G2 Pro | USB 35BB:A100 | 未知 | 音量、原有音效、灯光 | 音效需官方组件；灯光开关已验证切换与恢复，模式/RGB未在本次写入验证，只显示本工具保存值 |
| Logitech G305 LIGHTSPEED（preview） | 未实测；仅接收器 046D:C53F、配对槽 1，须同时确认 WPID 4074、HID++ 4.2 与 G305 精确名称 | 本机未知；上游公开版本不算本机验证 | 仅公开只读电量与身份查询，无设置写入 | 不支持 G304、其他接收器/多槽或蓝牙；尚未实测 Windows 电量、唤醒、拔插与并发，详见 [G305 范围](LOGITECH-G305.md) |

此表区分已有核心协议验证和新 DLL 的 ABI/打包检查；不把软件检查写成实机通过。连接模式或固件尚未单独确认时应明确保留未知。

Basilisk V3 Pro 有线预览仅匹配 `1532:00AA`、唯一 `MI_00` / 91 字节 Feature 集合，只读电量与充电状态。Windows 报告长度依据公开协议推导，尚无该型号本机验证；不包含 `00AB` 接收器、35K 变种、蓝牙或设置写入。详见[固定协议依据](RAZER-BASILISK-V3-PRO.md)。

A7 V3 Ultra+ 与 VGN F2 Master+ 包版本为 **0.4.4-preview.2**。A7 仅 `3837:1018` / `FF01:0001` / 64 字节输入输出，产品名必须精确匹配；F2 Master+ 仅 `3554:FB56/FB57` / `FF02` / 17 字节输入输出。两者只读电量与充电状态，尚未实机验证，不开放设置、改键、宏或灯光。固件兼容范围未知。详见 [A7 范围](MCHOSE-A7-V3.md)与 [F2 Master+ 范围](VGN-F2-MASTER-PLUS.md)。

0.4.5-preview.1 新增 **ATK X1 Pro Max**（有线 373B:1017 / 8K 接收器 373B:101B，CID/MID 必须 2,39）及 **Viper V2 Pro 有线**。两款均为公开实现适配、尚未实机验证；仅只读电量/充电状态，不提供设置写入。详见 [ATK 范围](ATK-X1-PRO-MAX.md)与 [Viper 范围](RAZER-VIPER-V2-PRO.md)。

0.4.6-preview.1 继续新增 **ATK A9 Plus Nearlink**（有线 373B:1115 / NearLink 接收器 373B:10C9，CID/MID 2,83）及 **Viper Ultimate 有线**（1532:007A）。只读电量/充电状态，未实机验证，不开放写入。详见 [A9 范围](ATK-A9-PLUS-NEARLINK.md)与 [Viper Ultimate 范围](RAZER-VIPER-ULTIMATE.md)。

0.4.7-preview.1 新增 **VGN F2 Pro Max**、**VGN F1 Pro** 的有线/接收器电量与充电状态，以及 **DeathAdder V2 Pro 有线**。均依据上游已有实现接入，移植后尚未实机复测，暂不提供设置写入；精确来源和范围见 [F2 Pro Max](VGN-F2-PRO-MAX.md)、[F1 Pro](VGN-F1-PRO.md)、[DeathAdder V2 Pro](RAZER-DEATHADDER-V2-PRO.md)。

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
