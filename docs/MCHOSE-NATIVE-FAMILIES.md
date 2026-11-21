# 迈从原生设置与本地模板

适用于外设盒 0.4.12-preview.4 和下列型号的 0.4.13-preview.52 预览插件。打开「插件管理」选择自己的型号，安装后在「设备调节」点选已连接的设备。多个同型号设备按所选接收器或有线设备分别操作。

## 磁轴键盘

点键盘图选择实体键。在对应页面设置触发深度、快速触发、DKS、SOCD、RS、MT、TGL、组合键；LT 与部分性能项依型号及固件开放。高级键可切换键位层。当前板载档在编辑期间变更时，需要重新读取。

| 有线键盘及布局版本 | USB VID:PID | 原厂可选实体键数 |
| --- | --- | ---: |
| [迈从 Ace 60](../drivers/mchose-ace-60) | `41E4:2101` | 61 |
| [迈从 Ace 60 Pro](../drivers/mchose-ace-60-pro) | `41E4:2103` | 61 |
| [迈从 Ace 60 Pro ISO FR](../drivers/mchose-ace-60-pro-iso-fr) | `3837:3040` | 62 |
| [迈从 Ace 60 Pro Nordic](../drivers/mchose-ace-60-pro-nordic) | `3837:3002` | 62 |
| [迈从 Ace 60X I](../drivers/mchose-ace-60x-i) | `41E4:2126` | 61 |
| [迈从 Ace 60X II](../drivers/mchose-ace-60x-ii) | `41E4:2112` | 61 |
| [迈从 Ace 68 Air 2](../drivers/mchose-ace-68-air-2) | `3837:300A` | 72 |
| [迈从 Ace 68 Air II](../drivers/mchose-ace-68-air-ii) | `41E4:2120` | 68 |
| [迈从 Ace 68 Air III](../drivers/mchose-ace-68-air-iii) | `41E4:2132` | 68 |
| [迈从 Ace 68 GT](../drivers/mchose-ace-68-gt) | `3837:3007` | 69 |
| [迈从 Ace 68 I](../drivers/mchose-ace-68-i) | `41E4:2114` | 68 |
| [迈从 Ace 68 III](../drivers/mchose-ace-68-iii) | `3837:3003` | 68 |
| [迈从 Ace 68 Pro](../drivers/mchose-ace-68-pro) | `41E4:2116` | 68 |
| [迈从 Ace 68 Turbo 16K](../drivers/mchose-ace-68-turbo-16k) | `3837:3026` | 69 |
| [迈从 Ace 68 Turbo 8K](../drivers/mchose-ace-68-turbo-8k) | `3837:3028` | 70 |
| [迈从 Ace 68 V2 III](../drivers/mchose-ace-68-v2-iii) | `3837:3024` | 71 |
| [迈从 Ace 75 16K](../drivers/mchose-ace-75-16k) | `3837:301D` | 86 |
| [迈从 Ace 75 8K](../drivers/mchose-ace-75-8k) | `3837:303C` | 86 |
| [迈从 God 60](../drivers/mchose-god-60) | `3837:3020` | 66 |
| [迈从 GT 75](../drivers/mchose-gt-75) | `3837:302D` | 83 |
| [迈从 Jet 75 I](../drivers/mchose-jet-75-i) | `41E4:2118` | 80 |
| [迈从 Jet 75 II](../drivers/mchose-jet-75-ii) | `41E4:211A` | 80 |
| [迈从 Jet 75 III](../drivers/mchose-jet-75-iii) | `3837:3008` | 80 |
| [迈从 Mix 87 I](../drivers/mchose-mix-87-i) | `41E4:2122` | 87 |
| [迈从 Mix 87 III](../drivers/mchose-mix-87-iii) | `3837:300D` | 87 |
| [迈从 Zero75X](../drivers/mchose-zero-75x) | `41E4:211C` | 80 |

## 耳机与音箱

选择「播放」或「麦克风」后，可选择此设备实际公开的 Windows 端点。板载音效、灯光和屏幕页按具体型号开放。V9 Pro 2 与 R9 的三个十段 EQ 槽均可写入，也可放入本地模板。

| 型号 | 连接 | 原生功能 |
| --- | --- | --- |
| [迈从 V9 Pro](../drivers/mchose-v9-pro) | `291D:385D` | 依接收器固件开放的板载音效模式 |
| [迈从 X9](../drivers/mchose-x9) | `3837:6045` | 依接收器固件开放的板载音效模式 |
| [迈从 S9 Pro](../drivers/mchose-s9-pro) | `291D:385D` | Windows 播放和麦克风控制 |
| [迈从 G9 Pro](../drivers/mchose-g9-pro) | `291D:385D` | 依接收器固件开放的板载音效模式 |
| [迈从 V9](../drivers/mchose-v9) | `291D:385D` | Windows 播放和麦克风控制 |
| [迈从 V9 Pro 2](../drivers/mchose-v9-pro-2) | `3837:6011` | 三套十段板载 EQ、提示音量与自动关机 |
| [迈从 R9](../drivers/mchose-r9) | `3837:6013` | 三套十段板载 EQ、提示音量与自动关机 |
| [迈从 V9 Turbo](../drivers/mchose-v9-turbo) | `3837:600D` | 声音模式、提示音量及定时关机 |
| [迈从 V9 Turbo+](../drivers/mchose-v9-turbo-plus) | `3837:600A` | 声音模式、提示音量及定时关机 |
| [迈从 G9](../drivers/mchose-g9) | `3837:8080` | Windows 播放和麦克风控制 |
| [迈从 G20](../drivers/mchose-g20) | `0E6A:0111` | 板载音效模式和语音提示 |
| [迈从 G20 Pro](../drivers/mchose-g20-pro) | `0E6A:0111` | 板载音效模式和语音提示 |
| [迈从 K20](../drivers/mchose-k20) | `3837:60C4` | 播放、板载音效、语音提示、灯效模式、周期、亮度和颜色 |
| [迈从 K20 Pro](../drivers/mchose-k20-pro) | `3837:6003` | 播放、板载音效、语音提示、灯效模式、周期、亮度和颜色 |
| [迈从 K20 GT](../drivers/mchose-k20-gt) | `3837:60C6` | 播放、板载音效、语音提示、灯效模式、周期、亮度和颜色。还可调屏幕开关、主题与颜色 |

[音频型号、固件门槛、板载与系统音效的区别](../crates/protocol-mchose-audio/UPSTREAM.md)

## 鼠标模板

以下鼠标共用 0x4D 协议，均可保存本机模板、导入原版 M HUB 文件和应用到当前板载档。每个接收器按机身自报身份匹配。支持的设置与宏以该型号响应为准。

[迈从 A5 V3 Pro](../drivers/mchose-a5-v3-pro)、[迈从 A5 V3 Ultra](../drivers/mchose-a5-v3-ultra)、[迈从 A5 V3 Ultra+(3955)](../drivers/mchose-a5-v3-ultra-plus-3955)、[迈从 A7 V3](../drivers/mchose-a7-v3)、[迈从 A7 V3 Magnetic](../drivers/mchose-a7-v3-magnetic)、[迈从 A7 V3 Pro](../drivers/mchose-a7-v3-pro)、[迈从 A7 V3 Pro+](../drivers/mchose-a7-v3-pro-plus)、[迈从 A7 V3 Ultra](../drivers/mchose-a7-v3-ultra)、[迈从 A7 V3-X](../drivers/mchose-a7-v3-x)、[迈从 K5 Pro](../drivers/mchose-k5-pro)、[迈从 K5 Ultra](../drivers/mchose-k5-ultra)、[迈从 K5 V2](../drivers/mchose-k5-v2)、[迈从 K7 V2 Pro](../drivers/mchose-k7-v2-pro)、[迈从 K7 V2 Pro+](../drivers/mchose-k7-v2-pro-plus)、[迈从 K7 V2 Ultra](../drivers/mchose-k7-v2-ultra)、[迈从 K7 V2 Ultra+](../drivers/mchose-k7-v2-ultra-plus)、[迈从 R7 Ultra](../drivers/mchose-r7-ultra)。

## 验证与来源

配套主程序用原生 Windows 控件测试了实体键选择、设备步长、窗口缩放后保留编辑值和 EQ 频段。驱动按固定的 M HUB 实现和原始布局核对，以软件样本验证协议与恢复。本项目在其他 USB 音频设备上只读验证了 Windows 播放端点和设备筛选。上述迈从磁轴键盘与音频型号的新增板载行为尚无本批实机写入记录。

[固定 M HUB 前端](https://www.mchose.com.cn/assets/main-DIt5vAcV.js) · [磁轴功能、布局及手动校准范围](../crates/protocol-mchose-cz/UPSTREAM.md) · [CZ 原厂键盘编辑器](https://www.mchose.com.cn/cizhou/) · [开源协议和型号归属](../THIRD_PARTY_LICENSES/protocol-facts.md)
