# 按厂商查找外设

共 51 个品牌、285 个插件条目。请在外设盒驱动商店选择对应通道，搜索型号并安装。

[8BitDo](#vendor-8bitdo) · [AJAZZ](#vendor-ajazz) · [AKG](#vendor-akg) · [ANGRY MIAO](#vendor-angry-miao) · [Astro](#vendor-astro) · [ASUS ROG](#vendor-asus-rog) · [ATK](#vendor-atk) · [ATTACK SHARK](#vendor-attack-shark) · [Audeze](#vendor-audeze) · [AULA](#vendor-aula) · [BETOP](#vendor-betop) · [CHERRY](#vendor-cherry) · [Corsair](#vendor-corsair) · [DARKRIM](#vendor-darkrim) · [DARMOSHARK](#vendor-darmoshark) · [DELUX](#vendor-delux) · [EWEADN](#vendor-eweadn) · [FUHLEN](#vendor-fuhlen) · [G-Wolves](#vendor-g-wolves) · [GameSir](#vendor-gamesir) · [GLORIOUS](#vendor-glorious) · [Haute42](#vendor-haute42) · [HECATE](#vendor-hecate) · [HITSCAN](#vendor-hitscan) · [HyperX](#vendor-hyperx) · [INPHIC](#vendor-inphic) · [Jabra](#vendor-jabra) · [JBL](#vendor-jbl) · [KEYCHRON](#vendor-keychron) · [LAMZU](#vendor-lamzu) · [Lenovo](#vendor-lenovo) · [LINGBAO](#vendor-lingbao) · [LOFREE](#vendor-lofree) · [Logitech](#vendor-logitech) · [MCHOSE](#vendor-mchose) · [NINJUTSO](#vendor-ninjutso) · [Nintendo](#vendor-nintendo) · [NZXT](#vendor-nzxt) · [Poly](#vendor-poly) · [Pulsar](#vendor-pulsar) · [Rapoo](#vendor-rapoo) · [Razer](#vendor-razer) · [ROYAL KLUDGE](#vendor-royal-kludge) · [SCYROX](#vendor-scyrox) · [Sony](#vendor-sony) · [STEAM](#vendor-steam) · [SteelSeries](#vendor-steelseries) · [THUNDEROBOT](#vendor-thunderobot) · [VGN](#vendor-vgn) · [VXE](#vendor-vxe) · [WLMOUSE](#vendor-wlmouse)

型号、连接方式和功能由各包独立声明。点开型号可查看源文件；验证记录只覆盖该包描述的连接方式与功能。接口匹配仍可能命中多个候选，由驱动商店区分。其他厂商的协议族可来自共享实现，未经单独确认的型号保留独立条目。

维护及完整型号查询见[贡献指南](../CONTRIBUTING.md#vendor-research)。结构化索引见 [vendors.json](vendors.json)。
原厂登记型号和上游协议规格可在[厂商研究资料](upstream/vigil-watt/README.md)中查阅。

<a id="vendor-8bitdo"></a>

## 8BitDo

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-8bitdo-pro2 | [8BitDo Pro 2 / SN30 Pro / SF30 Pro](../drivers/eightbitdo-pro-2/UPSTREAM.md) | 2DC8:6000 2DC8:6100 2DC8:6001 2DC8:6101 2DC8:6003 2DC8:6006 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-8bitdo-pro3 | [8BitDo Pro 3](../drivers/eightbitdo-pro-3) | 2DC8:6009 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-eightbitdo-ultimate-2 | [8BitDo Ultimate 2 Wireless](../drivers/eightbitdo-ultimate-2-wireless) | 2DC8:6012 | 电量 | 预览 | 接口匹配 | 公开实现 |

<a id="vendor-ajazz"></a>

## AJAZZ

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| ajazz-ak870 | [AJAZZ AK870](../drivers/ajazz-ak870/UPSTREAM.md) | 3151:4011 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-ajazz-aj159-apex | [AJAZZ AJ159 APEX (2.4G 8K)](../drivers/ajazz-aj159-apex) | 3151:5007 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-ajazz-aj179-pro | [AJAZZ AJ179 PRO](../drivers/ajazz-aj179-pro) | 3151:4026 3151:402D | 电量、设置、改键 | 预览 | 运行时核对 | 上游实机 |

<a id="vendor-akg"></a>

## AKG

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-akg-n9-hybrid | [AKG N9 Hybrid](../drivers/akg-n9-hybrid/README.md) | 0ECB:20E1 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-angry-miao"></a>

## ANGRY MIAO

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| am-infinity-8k | [AM Infinity 8K](../drivers/am-infinity-8k/UPSTREAM.md) | 3151:5007 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-astro"></a>

## Astro

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-astro-a50 | [Astro A50 第五代](../drivers/astro-a50-gen5) | 046D:0B1C | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-astro-a50 | [ASTRO A50 Gen 4](../drivers/logitech-astro-a50-gen4) | 9886:002C | 电量 | 预览 | 接口匹配 | 上游验证 |

<a id="vendor-asus-rog"></a>

## ASUS ROG

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| asus-rog-azoth-extreme | [ROG Azoth Extreme](../drivers/asus-rog-azoth-extreme) | 0B05:1ACE | 电量 | 预览 | 手动认领 | 上游实机 |
| asus-rog-scope-ii-96-omni | [ROG Strix Scope II 96 Omni](../drivers/asus-rog-scope-ii-96-omni/UPSTREAM.md) | 0B05:1ACE | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-asus-rog-gladius-iii-wireless | [ROG Gladius III Wireless AimPoint 36K](../drivers/asus-rog-gladius-iii-wireless/PROVENANCE.md) | 0B05:1A72 0B05:1A70 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-asus-rog-mice | [ROG Chakram X](../drivers/asus-rog-chakram-x) | 0B05:1A1A 0B05:1A18 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-asus-rog-mice | [ASUS ROG Gladius III Wireless（初代）](../drivers/asus-rog-gladius-iii-legacy/PROVENANCE.md) | 0B05:197F 0B05:197D | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-asus-rog-mice | [ASUS ROG Harpe Ace Aim Lab Edition](../drivers/asus-rog-harpe-ace-aim-lab/PROVENANCE.md) | 0B05:1A94 0B05:1A92 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-asus-rog-mice | [ROG Keris Wireless](../drivers/asus-rog-keris-wireless) | 0B05:1960 0B05:195E | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-asus-rog-mice | [ASUS ROG Pugio II](../drivers/asus-rog-pugio-ii/PROVENANCE.md) | 0B05:1908 0B05:1906 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-asus-rog-mice | [ASUS ROG Spatha X](../drivers/asus-rog-spatha-x/PROVENANCE.md) | 0B05:1979 0B05:1977 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-asus-rog-mice | [ASUS ROG Strix Impact II Wireless](../drivers/asus-rog-strix-impact-ii-wireless/PROVENANCE.md) | 0B05:1949 0B05:1947 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-atk"></a>

## ATK

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| atk-a9-air-master-plus | [ATK A9 Air 大师+](../drivers/atk-a9-air-master-plus/UPSTREAM.md) | 37BB:1278 | 电量 | 预览 | 接口匹配 | 上游实机 |
| atk-a98-pro | [ATK A98 Pro](../drivers/atk-a98-pro/UPSTREAM.md) | 37BB:1152 | 电量 | 预览 | 接口匹配 | 上游实机 |
| atk-f1-pro-max | [ATK F1 Pro Max](../drivers/atk-f1-pro-max/PROVENANCE.md) | 37BB:1015 37BB:101A 37BB:101B | 电量、设置 | 预览 | 运行时核对 | 上游实机 |
| atk-n9-v2-ultimate | [ATK N9 V2 Ultimate](../drivers/atk-n9-v2-ultimate/UPSTREAM.md) | 37BB:1262 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-atk | [ATK X1 Pro Max](../drivers/atk-x1-pro-max) | 37BB:1017 37BB:101B | 电量、设置 | 预览 | 运行时核对 | 公开实现 |
| protocol-atk-a9-plus-nearlink | [ATK A9 Plus Nearlink](../drivers/atk-a9-plus-nearlink) | 37BB:1115 37BB:10C9 | 电量、设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-atk-f1-battery | [ATK F1 Ultimate 2.0](../drivers/atk-f1-ultimate-2-0/UPSTREAM.md) | 37BB:11D9 37BB:11E4 | 电量 | 预览 | 接口匹配 | 公开实现 |
| protocol-atk-f1-battery | [ATK F1 V3 ULTIMATE+](../drivers/atk-f1-v3-ultimate-plus/UPSTREAM.md) | 37BB:1303 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-attack-shark"></a>

## ATTACK SHARK

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-attack-shark-x6 | [Attack Shark X6](../drivers/attack-shark-x6) | 1D57:FA60 | 电量 | 预览 | 手动认领 | 上游实机 |

<a id="vendor-audeze"></a>

## Audeze

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-audeze-maxwell | [奥蒂兹 Maxwell](../drivers/audeze-maxwell) | 3329:4B19 3329:4B1A | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-audeze-maxwell | [Audeze Maxwell 2](../drivers/audeze-maxwell2) | 3329:4B29 3329:4B28 | 电量 | 预览 | 接口匹配 | 上游验证 |

<a id="vendor-aula"></a>

## AULA

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| aula-f108-pro | [AULA F108 Pro](../drivers/aula-f108-pro/UPSTREAM.md) | 05AC:024F | 电量 | 预览 | 接口匹配 | 上游实机 |
| aula-g9 | [AULA G9](../drivers/aula-g9/UPSTREAM.md) | 38A6:0030 | 电量 | 预览 | 接口匹配 | 上游实机 |
| aula-hero-68-air | [AULA HERO 68 Air](../drivers/aula-hero-68-air/UPSTREAM.md) | 372E:106C | 电量 | 预览 | 接口匹配 | 上游实机 |
| aula-l9-ultra | [AULA L9 Ultra](../drivers/aula-l9-ultra/UPSTREAM.md) | 38A6:28FD | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-aula-f99-pro | [AULA F99 Pro](../drivers/aula-f99-pro) | 3554:FA09 | 电量 | 预览 | 手动认领 | 上游实机 |

<a id="vendor-betop"></a>

## BETOP

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| 鲲鹏 · 0x15 状态 | [BETOP Kunpeng 20](../drivers/betop-kunpeng20/UPSTREAM.md) | 20BC:5159 20BC:5158 | 电量 | 预览 | 接口匹配 | 上游实机 |
| 宙斯 · 0x04 按键状态 | [BETOP T9 / 北通宙斯 2](../drivers/betop-t9/UPSTREAM.md) | 20BC:5073 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-cherry"></a>

## CHERRY

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| cherry-pixiu75 | [CHERRY PIXIU75](../drivers/cherry-pixiu75/UPSTREAM.md) | 046A:01E4 046A:01E2 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-corsair"></a>

## Corsair

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-corsair-dark-core-rgb-pro-se | [Corsair Dark Core RGB Pro SE](../drivers/corsair-dark-core-rgb-pro-se/UPSTREAM.md) | 1B1C:1B7F | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-corsair-headsets | [海盗船 Virtuoso RGB Wireless SE / XT](../drivers/corsair-virtuoso-rgb-se-xt) | 1B1C:0A3E 1B1C:0A3D 1B1C:0A40 1B1C:0A64 1B1C:0A62 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-corsair-headsets | [Corsair Void Pro / RGB 系列](../drivers/corsair-void-rich) | 1B1C:1B1C 1B1C:1B27 1B1C:0A14 1B1C:0A16 1B1C:0A17 1B1C:0A1D 1B1C:0A1A 1B1C:1B2A 1B1C:1B23 1B1C:1B29 1B1C:0A55 1B1C:0A51 1B1C:0A52 1B1C:0A38 1B1C:0A4F 1B1C:0A2B 1B1C:0A75 1B1C:0A56 | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-corsair-headsets | [海盗船 Void v2 / Virtuoso Max / HS80 Max](../drivers/corsair-void-v2-family) | 1B1C:2A08 1B1C:2A02 1B1C:0A97 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-darkrim"></a>

## DARKRIM

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| razer-darkrim-shadow | [DARKRIM SHADOW](../drivers/razer-darkrim-shadow/PROVENANCE.md) | 3654:6901 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-darmoshark"></a>

## DARMOSHARK

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| darmoshark-m3 | [Darmoshark M3](../drivers/darmoshark-m3/UPSTREAM.md) | 248A:FF30 | 电量 | 预览 | 运行时核对 | 上游实机 |

<a id="vendor-delux"></a>

## DELUX

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| delux-m900-pro | [DELUX M900 Pro](../drivers/delux-m900-pro/UPSTREAM.md) | 1D57:FA65 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-eweadn"></a>

## EWEADN

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| eweadn-s9-series | [EWEADN S9 Pro/Max/Ultra](../drivers/eweadn-s9-series/UPSTREAM.md) | 372E:1026 | 电量 | 预览 | 接口匹配 | 上游实机 |
| eweadn-x87 | [EWEADN X87](../drivers/eweadn-x87/UPSTREAM.md) | 0C45:FEFE | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-fuhlen"></a>

## FUHLEN

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| fuhlen-n87 | [Fuhlen N87](../drivers/fuhlen-n87/UPSTREAM.md) | 3151:4011 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-g-wolves"></a>

## G-Wolves

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| gwolves-hsk-pro-2 | [G-Wolves HSK Pro 2.0](../drivers/gwolves-hsk-pro-2/PROVENANCE.md) | 33E4:6817 33E4:6808 | 电量 | 预览 | 接口匹配 | 代码适配 |
| gwolves-hts-plus-ace | [G-Wolves HTS Plus ACE](../drivers/gwolves-hts-plus-ace/PROVENANCE.md) | 33E4:5403 33E4:5404 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-gwolves-old | [G-Wolves Fenrir Max](../drivers/gwolves-fenrir-max/UPSTREAM.md) | 33E4:3717 33E4:3708 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-gwolves-old | [G-Wolves HSK Plus ACE](../drivers/gwolves-hsk-plus-ace/UPSTREAM.md) | 33E4:5903 33E4:5904 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-gwolves-old | [G-Wolves HSK Pro ACE](../drivers/gwolves-hsk-pro-ace/UPSTREAM.md) | 33E4:5803 33E4:5804 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-gwolves-old | [G-Wolves HT-S2](../drivers/gwolves-ht-s2/UPSTREAM.md) | 33E4:7913 33E4:7904 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-gwolves-old | [G-Wolves HTS Ultra](../drivers/gwolves-hts-ultra/UPSTREAM.md) | 33E4:5317 33E4:5308 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-gwolves-old | [G-Wolves VUK](../drivers/gwolves-vuk/UPSTREAM.md) | 33E4:3917 33E4:3908 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-gamesir"></a>

## GameSir

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| gamesir-g7-pro-ce | [GameSir G7 Pro CE](../drivers/gamesir-g7-pro-ce/PROVENANCE.md) | 3537:10B9 3537:0575 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-glorious"></a>

## GLORIOUS

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-glorious-mow | [Glorious Model D- Wireless](../drivers/glorious-model-d-minus-wireless/UPSTREAM.md) | 258A:2025 258A:2014 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-glorious-mow | [Glorious Model D Wireless](../drivers/glorious-model-d-wireless/UPSTREAM.md) | 258A:2023 258A:2012 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-glorious-mow | [Glorious Model O- Wireless](../drivers/glorious-model-o-minus-wireless/UPSTREAM.md) | 258A:2024 258A:2013 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-glorious-mow | [Glorious Model O Wireless](../drivers/glorious-model-o-wireless/UPSTREAM.md) | 258A:2022 258A:2011 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-haute42"></a>

## Haute42

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-haute42-4ds | [Haute42 Adapter 4DS](../drivers/haute42-adapter-4ds/README.md) | 054C:0CE6 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-hecate"></a>

## HECATE

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| core-hecate | [漫步者 G2 Pro](../drivers/hecate-g2-pro) | 35BB:A100 | volume、effects、lighting、灯光开关 | 稳定 | 接口匹配 | 本项目实机 |
| protocol-hecate-g3m-pro-upgrade | [HECATE D3FB 接收器（G3M Pro Upgrade 型号未确认）](../drivers/hecate-g3m-pro-upgrade/README.md) | 35BB:D3FB | 电量 | 预览 | 接口匹配 | 公开实现 |
| protocol-hecate-g5m-pro | [HECATE G5M Pro](../drivers/hecate-g5m-pro/README.md) | 35BB:D00B | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-hitscan"></a>

## HITSCAN

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-pulsar-17 | [Hitscan Hyperlight](../drivers/hitscan-hyperlight/UPSTREAM.md) | 3770:0200 3770:0100 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-hyperx"></a>

## HyperX

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| hyperx-cloud-ii-wireless-kingston | [HyperX Cloud II Wireless (Kingston)](../drivers/hyperx-cloud-ii-wireless-kingston) | 0951:1718 | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-hyperx-cloud | [极度未知 Cloud Alpha 2](../drivers/hyperx-cloud-alpha-2) | 03F0:08BE | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hyperx-cloud | [极度未知 Cloud II Core Wireless](../drivers/hyperx-cloud-ii-core-wireless) | 03F0:0995 03F0:0795 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hyperx-cloud | [极度未知 Cloud II Wireless](../drivers/hyperx-cloud-ii-wireless) | 03F0:0696 03F0:018B | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-hyperx-cloud | [极度未知 Cloud III S Wireless](../drivers/hyperx-cloud-iii-s-wireless) | 03F0:02CC 03F0:06BE | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hyperx-cloud | [极度未知 Cloud III Wireless](../drivers/hyperx-cloud-iii-wireless) | 03F0:05B7 03F0:0C9D | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-hyperx-cloud-flight-2 | [HyperX Cloud Flight 2](../drivers/hyperx-cloud-flight-2/README.md) | 03F0:0AC1 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-inphic"></a>

## INPHIC

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| inphic-zhen | [INPHIC ZHEN](../drivers/inphic-zhen/UPSTREAM.md) | 1D57:FA65 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-jabra"></a>

## Jabra

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-jabra-gnp | [Jabra Evolve2 65 Flex (USB)](../drivers/jabra-evolve2-65-flex) | 0B0E:2519 | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-jabra-gnp | [Jabra Link 380（已配对耳机）](../drivers/jabra-link-380) | 0B0E:24C7 | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-jabra-gnp | [Jabra Link 390（已配对耳机）](../drivers/jabra-link-390) | 0B0E:2E57 | 电量 | 预览 | 接口匹配 | 上游验证 |

<a id="vendor-jbl"></a>

## JBL

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-jbl-quantum | [JBL Quantum 910 Wireless](../drivers/jbl-quantum-910) | 0ECB:2088 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-keychron"></a>

## KEYCHRON

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-keychron | [Keychron M5](../drivers/keychron-m5) | 3434:D048 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-keychron | [Keychron Ultra-Link 8K](../drivers/keychron-ultralink-8k) | 3434:D028 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-lamzu"></a>

## LAMZU

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-lamzu-maya-x | [LAMZU Maya X](../drivers/lamzu-maya-x/UPSTREAM.md) | 373E:001E 373E:001C | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-lenovo"></a>

## Lenovo

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| lenovo-wireless-voip | [Lenovo Wireless VoIP Headset](../drivers/lenovo-wireless-voip) | 17EF:A07D | 电量 | 预览 | 接口匹配 | 上游验证 |

<a id="vendor-lingbao"></a>

## LINGBAO

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| lingbao-m5pro-max | [LINGBAO M5PRO Max](../drivers/lingbao-m5pro-max/PROVENANCE.md) | 3151:402D | 电量 | 预览 | 手动认领 | 上游实机 |

<a id="vendor-lofree"></a>

## LOFREE

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-lofree | [Lofree Hyzen](../drivers/lofree-hyzen) | 388D:0025 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-logitech"></a>

## Logitech

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-hidpp | [罗技 Craft Advanced Keyboard 蓝牙版](../drivers/logitech-craft-keyboard) | 046D:B350 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 ERGO M575 Trackball 蓝牙版](../drivers/logitech-ergo-m575) | 046D:B027 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 G Pro X Superlight 2](../drivers/logitech-g-pro-x-superlight-2/UPSTREAM.md) | 046D:C547 046D:C54D 046D:C09B | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 G102](../drivers/logitech-g102/UPSTREAM.md) | 046D:C539 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 G304 LIGHTSPEED](../drivers/logitech-g304/UPSTREAM.md) | 046D:C53F | 电量、板载模式、配置与 DPI 档位切换 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 G305 LIGHTSPEED](../drivers/logitech-g305) | 046D:C53F | 电量、设置、板载模式、配置与 DPI 档位切换 | 预览 | 运行时核对 | 公开实现 |
| protocol-hidpp | [罗技 G502 LIGHTSPEED](../drivers/logitech-g502-lightspeed) | 046D:C539 046D:C547 | 电量、板载模式、配置与 DPI 档位切换 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 G502 X PLUS](../drivers/logitech-g502-x-plus) | 046D:C539 046D:C547 | 电量、板载模式、配置与 DPI 档位切换 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 G515 LIGHTSPEED TKL 蓝牙版](../drivers/logitech-g515-ls-tkl) | 046D:B389 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 G604 LIGHTSPEED 蓝牙版](../drivers/logitech-g604) | 046D:B024 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 G613 Wireless Keyboard 蓝牙版](../drivers/logitech-g613) | 046D:B34F | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [Logitech HID++ 通用电量](../drivers/logitech-hidpp-generic/UPSTREAM.md) | 046D:C52B 046D:C532 046D:C539 046D:C53A 046D:C53D 046D:C53F 046D:C541 046D:C545 046D:C547 046D:C54D 046D:C548 046D:C09B | 电量 | 预览 | 手动认领 | 上游验证 |
| protocol-hidpp | [罗技 K230 无线键盘](../drivers/logitech-k230) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 K360 无线键盘](../drivers/logitech-k360) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 K380 Multi-Device Keyboard 蓝牙版](../drivers/logitech-k380) | 046D:B342 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 K400 Plus 无线触控键盘](../drivers/logitech-k400-plus) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 K600 TV Keyboard 蓝牙版](../drivers/logitech-k600-tv) | 046D:B353 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 K780 Multi-Device Keyboard 蓝牙版](../drivers/logitech-k780) | 046D:B33B | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 K850 Performance Keyboard 蓝牙版](../drivers/logitech-k850) | 046D:B34D | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 LIFT Vertical Ergonomic Mouse 蓝牙版](../drivers/logitech-lift) | 046D:B031 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 LIFT for Business 蓝牙版](../drivers/logitech-lift-for-business) | 046D:B033 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 M185 / M235 / M310 无线鼠标](../drivers/logitech-m185-m235-m310) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 M215 第二代无线鼠标](../drivers/logitech-m215) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 M325 无线鼠标](../drivers/logitech-m325) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 M585 / M590 蓝牙版](../drivers/logitech-m585-m590) | 046D:B01B | 电量 | 预览 | 运行时核对 | 代码适配 |
| protocol-hidpp | [罗技 M720 Triathlon 蓝牙版](../drivers/logitech-m720-triathlon) | 046D:B015 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 M750M](../drivers/logitech-m750m/UPSTREAM.md) | 046D:C548 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MK270 无线键盘](../drivers/logitech-mk270) | 046D:C52B 046D:C532 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MK710](../drivers/logitech-mk710) | 046D:C52B | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Anywhere 2S 蓝牙版](../drivers/logitech-mx-anywhere-2s) | 046D:B01A | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Anywhere 3 蓝牙版](../drivers/logitech-mx-anywhere-3) | 046D:B025 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Anywhere 3 for Business 蓝牙版](../drivers/logitech-mx-anywhere-3-business) | 046D:B02D | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Ergo Trackball 蓝牙版](../drivers/logitech-mx-ergo) | 046D:B01D | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Keys Keyboard 蓝牙版](../drivers/logitech-mx-keys) | 046D:B35B | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Keys for Business 蓝牙版](../drivers/logitech-mx-keys-business) | 046D:B363 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Keys Mini 蓝牙版](../drivers/logitech-mx-keys-mini) | 046D:B369 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Keys S 蓝牙版](../drivers/logitech-mx-keys-s) | 046D:B378 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Master 2S 蓝牙版](../drivers/logitech-mx-master-2s) | 046D:B019 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Master 3 蓝牙版](../drivers/logitech-mx-master-3) | 046D:B023 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Master 3 for Business 蓝牙版](../drivers/logitech-mx-master-3-business) | 046D:B028 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Master 3S 蓝牙版](../drivers/logitech-mx-master-3s) | 046D:B034 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Master 4 蓝牙版](../drivers/logitech-mx-master-4) | 046D:B042 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Master (B012) 蓝牙版](../drivers/logitech-mx-master-b012) | 046D:B012 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Master (B01E) 蓝牙版](../drivers/logitech-mx-master-b01e) | 046D:B01E | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 MX Mechanical 蓝牙版](../drivers/logitech-mx-mechanical) | 046D:B366 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Mechanical Mini 蓝牙版](../drivers/logitech-mx-mechanical-mini) | 046D:B367 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 MX Vertical 蓝牙版](../drivers/logitech-mx-vertical) | 046D:B020 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 Pebble M350 蓝牙版](../drivers/logitech-pebble-m350) | 046D:B021 | 电量 | 预览 | 运行时核对 | 上游实机 |
| protocol-hidpp | [罗技 POP Keys 蓝牙版](../drivers/logitech-pop-keys) | 046D:B365 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 Signature M550 蓝牙版](../drivers/logitech-signature-m550) | 046D:B02B | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-hidpp | [罗技 Signature M650 L 蓝牙版](../drivers/logitech-signature-m650-l) | 046D:B02A | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-logitech-bolt-mx-master-3s | [Logitech Bolt + MX Master 3S](../drivers/logitech-bolt-mx-master-3s/README.md) | 046D:C548 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-logitech-g522 | [Logitech G522 LIGHTSPEED](../drivers/logitech-g522-lightspeed/UPSTREAM.md) | 046D:0B18 | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-logitech-gseries-headset | [罗技 G533](../drivers/logitech-g533) | 046D:C52B 046D:C532 046D:C52E 046D:C531 046D:C534 046D:C535 046D:C537 046D:C539 046D:C53A 046D:C53D 046D:C53F 046D:C541 046D:C545 046D:C547 046D:C54D 046D:0A66 | 电量 | 预览 | 运行时核对 | 代码适配 |
| protocol-logitech-gseries-headset | [罗技 G535](../drivers/logitech-g535) | 046D:C52B 046D:C532 046D:C52E 046D:C531 046D:C534 046D:C535 046D:C537 046D:C539 046D:C53A 046D:C53D 046D:C53F 046D:C541 046D:C545 046D:C547 046D:C54D 046D:0AC4 | 电量 | 预览 | 运行时核对 | 代码适配 |
| protocol-logitech-gseries-headset | [罗技 G733](../drivers/logitech-g733) | 046D:C52B 046D:C532 046D:C52E 046D:C531 046D:C534 046D:C535 046D:C537 046D:C539 046D:C53A 046D:C53D 046D:C53F 046D:C541 046D:C545 046D:C547 046D:C54D 046D:0AB5 046D:0AFE 046D:0B1F | 电量 | 预览 | 运行时核对 | 代码适配 |
| protocol-logitech-gseries-headset | [Logitech G930](../drivers/logitech-g930) | 046D:0A1F | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-logitech-gseries-headset | [Logitech G933](../drivers/logitech-g933) | 046D:0A5B | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-logitech-gseries-headset | [罗技 G935](../drivers/logitech-g935) | 046D:C52B 046D:C532 046D:C52E 046D:C531 046D:C534 046D:C535 046D:C537 046D:C539 046D:C53A 046D:C53D 046D:C53F 046D:C541 046D:C545 046D:C547 046D:C54D 046D:0A87 | 电量 | 预览 | 运行时核对 | 代码适配 |
| protocol-logitech-pro-x2 | [Logitech PRO X 2 LIGHTSPEED](../drivers/logitech-pro-x2-lightspeed/README.md) | 046D:0AF7 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-mchose"></a>

## MCHOSE

[迈从各代接收器与厂商研究记录](../docs/brands/MCHOSE.md)

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| A5 初代 · 取反 Feature | [迈从 A5 Ultra 初代](../drivers/mchose-a5-ultra) | 3837:100C | 电量、设置、改键、板载宏（实验）、software-macro-trigger | 稳定 | 接口匹配 | 本项目实机 |
| A5 Pro Max · 0x83 状态 | [MCHOSE A5 Pro Max](../drivers/mchose-a5-pro-max/UPSTREAM.md) | 2023:F013 2023:F015 2023:F019 | 电量 | 预览 | 接口匹配 | 上游实机 |
| A7 V3 · 0x4D 事务 | [MCHOSE A7 V3 Ultra+ 接收器](../drivers/mchose-a7-v3-ultra-plus) | 3837:1018 | 电量、设置、改键 | 预览 | 接口匹配 | 公开实现 |
| CZ 键盘 · 状态分块读取 | [MCHOSE G87 V2 · CZ 接收器](../drivers/mchose-g87-v2-cz/UPSTREAM.md) | 3837:3033 | 电量 | 预览 | 手动认领 | 上游实机 |
| Gmouse · AA 30 状态 | [迈从 G3 V2](../drivers/mchose-g3-v2/UPSTREAM.md) | 3837:4245 | 电量 | 预览 | 接口匹配 | 上游实机 |
| Gmouse · AA 30 状态 | [MCHOSE G7](../drivers/mchose-g7/UPSTREAM.md) | A8A5:2255 | 电量 | 预览 | 接口匹配 | 上游实机 |
| G87 键盘 · 20 字节 V1 会话 | [MCHOSE G87 V1](../drivers/mchose-g87-v1/UPSTREAM.md) | 41E4:2001 | 电量 | 预览 | 接口匹配 | 上游实机 |
| G87 键盘 · 20 字节 V1 会话 | [MCHOSE G87 V2](../drivers/mchose-g87-v2/UPSTREAM.md) | 3837:201B | 电量 | 预览 | 接口匹配 | 上游实机 |
| 无线耳机 · 55 65 状态 | [迈从 V9 Pro](../drivers/mchose-v9-pro) | 291D:385D | 电量 | 预览 | 接口匹配 | 公开实现 |
| 无线耳机 · 55 65 状态 | [MCHOSE X9 Wireless Headset](../drivers/mchose-x9) | 3837:6045 | 电量 | 预览 | 接口匹配 | 上游验证 |
| Compx 键盘 · 0x13 会话 | [迈从 G75 Pro](../drivers/mchose-g75-pro) | 3554:FA09 41E4:2001 | 电量、设置、改键、key-colors、software-macro-trigger | 稳定 | 接口匹配 | 本项目实机 |
| Compx 键盘 · 0x13 会话 | [MCHOSE K99 V2](../drivers/mchose-k99-v2) | 3554:FA09 | 电量 | 预览 | 手动认领 | 上游实机 |
| MagDock · 底座灯效 | [MCHOSE MagDock](../drivers/mchose-magdock) | 3837:1012 | 灯效编辑、灯光开关、流动方向 | 预览 | 接口匹配 | 上游实机 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A5 V3 Pro](../drivers/mchose-a5-v3-pro) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A5 V3 Ultra](../drivers/mchose-a5-v3-ultra) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A5 V3 Ultra+(3955)](../drivers/mchose-a5-v3-ultra-plus-3955) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A7 V3](../drivers/mchose-a7-v3) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A7 V3 Magnetic](../drivers/mchose-a7-v3-magnetic) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A7 V3 Pro](../drivers/mchose-a7-v3-pro) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A7 V3 Pro+](../drivers/mchose-a7-v3-pro-plus) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A7 V3 Ultra](../drivers/mchose-a7-v3-ultra) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 A7 V3-X](../drivers/mchose-a7-v3-x) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 K5 Pro](../drivers/mchose-k5-pro) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 K5 Ultra](../drivers/mchose-k5-ultra) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 K7 V2 Pro](../drivers/mchose-k7-v2-pro) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 K7 V2 Pro+](../drivers/mchose-k7-v2-pro-plus) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 K7 V2 Ultra](../drivers/mchose-k7-v2-ultra) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 K7 V2 Ultra+](../drivers/mchose-k7-v2-ultra-plus) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 0x4D 鼠标 · 通用 8K 接收器 | [迈从 R7 Ultra](../drivers/mchose-r7-ultra) | 3837:1018 | 电量、读配置回报率、读 DPI 档位配置、编辑 DPI 档位、设置回报率、指针设置、改键、板载宏（实验）、配置改名 | 预览 | 手动认领 | 公开实现 |
| 鼠标 · 取反 0x06 状态 | [MCHOSE A7 V2 Ultra](../drivers/mchose-a7-v2-ultra/UPSTREAM.md) | 3837:100B | 电量 | 预览 | 接口匹配 | 代码适配 |
| 鼠标 · 取反 0x06 状态 | [MCHOSE M7 Ultra](../drivers/mchose-m7-ultra/UPSTREAM.md) | 5253:1020 5253:0031 | 电量 | 预览 | 接口匹配 | 代码适配 |
| 鼠标 V2 · E2 状态上报 | [MCHOSE A7 V2 Pro](../drivers/mchose-a7-v2-pro/UPSTREAM.md) | 3837:100A | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-ninjutso"></a>

## NINJUTSO

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-ninjutso-sora-v2 | [Ninjutso Sora V2](../drivers/ninjutso-sora-v2/UPSTREAM.md) | 1915:AE12 1915:AE1C | 电量 | 预览 | 接口匹配 | 公开实现 |

<a id="vendor-nintendo"></a>

## Nintendo

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-switch-controller | [Nintendo Switch Pro Controller / Joy-Con](../drivers/nintendo-switch-pro/UPSTREAM.md) | 057E:2009 057E:2006 057E:2007 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-nzxt"></a>

## NZXT

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-nzxt-lift-elite-wireless | [NZXT Lift Elite Wireless](../drivers/nzxt-lift-elite-wireless/UPSTREAM.md) | 1E71:2101 1E71:2129 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-poly"></a>

## Poly

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| plantronics-bt600 | [Plantronics Voyager 8200 UC (BT600)](../drivers/plantronics-bt600) | 047F:02EE | 电量 | 预览 | 接口匹配 | 上游验证 |

<a id="vendor-pulsar"></a>

## Pulsar

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-pulsar-17 | [Pulsar X2 CrazyLight Mini](../drivers/pulsar-x2-crazylight-mini/UPSTREAM.md) | 3710:5406 3710:3414 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-pulsar-17 | [Pulsar X2 V2 Mini](../drivers/pulsar-x2-v2-mini/UPSTREAM.md) | 3554:F508 3554:F507 | 电量 | 预览 | 手动认领 | 公开实现 |
| pulsar-x2-v2 | [Pulsar X2 V2](../drivers/pulsar-x2-v2/PROVENANCE.md) | 3554:F508 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-rapoo"></a>

## Rapoo

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| VT · 报告 0x07 | [Rapoo VT3 II](../drivers/rapoo-vt3-ii/PROVENANCE.md) | 24AE:1412 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-razer"></a>

## Razer

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-razer | [Razer Basilisk V3 Pro](../drivers/razer-basilisk-v3-pro-wired) | 1532:00AA | 电量、设置、灯效编辑、灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer | [Razer BlackWidow V3 Mini HyperSpeed](../drivers/razer-blackwidow-v3-mini) | 1532:0271 1532:0258 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer | [Razer BlackWidow V3 Pro](../drivers/razer-blackwidow-v3-pro) | 1532:025C 1532:025A | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer | [Razer BlackWidow V4 Mini HyperSpeed](../drivers/razer-blackwidow-v4-mini) | 1532:02BA 1532:02B9 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer | [Razer BlackWidow V4 TKL HyperSpeed](../drivers/razer-blackwidow-v4-tkl-hyperspeed) | 1532:02D5 1532:02D7 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer | [炼狱蝰蛇 V3 专业版](../drivers/razer-deathadder-v3-pro) | 1532:00B6 1532:00B7 | 电量、设置、改键、software-macro-trigger、读 DPI 档位配置、编辑 DPI 档位、读配置回报率 | 预览 | 接口匹配 | 本项目实机 |
| protocol-razer | [炼狱蝰蛇 V3 专业版](../drivers/razer-deathadder-v3-pro) | 1532:00B6 1532:00B7 | 电量、设置、改键、software-macro-trigger | 稳定 | 接口匹配 | 本项目实机 |
| protocol-razer | [Razer DeathStalker V2 Pro](../drivers/razer-deathstalker-v2-pro) | 1532:0290 1532:0292 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer | [Razer DeathStalker V2 Pro TKL](../drivers/razer-deathstalker-v2-pro-tkl) | 1532:0296 1532:0298 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-basilisk-v3-wired | [Razer Basilisk V3](../drivers/razer-basilisk-v3-wired) | 1532:0099 | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-blackshark-2020 | [Razer BlackShark V2 Pro (2020)](../drivers/razer-blackshark-v2-pro-2020) | 1532:0528 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-blackshark-2023 | [Razer BlackShark V2 Pro (2023)](../drivers/razer-blackshark-v2-pro-2023/README.md) | 1532:0555 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-deathadder-v2-pro | [Razer DeathAdder V2 Pro](../drivers/razer-deathadder-v2-pro-wired) | 1532:007C | 电量、设置、灯效编辑、灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer BlackWidow Elite](../drivers/razer-blackwidow-elite) | 1532:0228 | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer BlackWidow V3](../drivers/razer-blackwidow-v3) | 1532:024E | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer BlackWidow V4](../drivers/razer-blackwidow-v4) | 1532:0287 | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Cynosa V2](../drivers/razer-cynosa-v2) | 1532:025E | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Huntsman Elite](../drivers/razer-huntsman-elite) | 1532:0226 | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Huntsman Mini](../drivers/razer-huntsman-mini) | 1532:0257 | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Huntsman Tournament Edition](../drivers/razer-huntsman-tournament-edition) | 1532:0243 | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Huntsman V2](../drivers/razer-huntsman-v2) | 1532:026C | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Huntsman V2 TKL](../drivers/razer-huntsman-v2-tkl) | 1532:026B | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-keyboard-light | [Razer Ornata V3](../drivers/razer-ornata-v3) | 1532:02A1 | 灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-kraken-v4 | [Razer Kraken V4](../drivers/razer-kraken-v4/README.md) | 1532:056C | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-naga-x | [Razer Naga X](../drivers/razer-naga-x) | 1532:0096 | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-pa | [Razer Barracuda Pro](../drivers/razer-barracuda-pro) | 1532:053A | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-viper-ultimate | [Razer Viper Ultimate](../drivers/razer-viper-ultimate-wired) | 1532:007A | 电量、设置、灯效编辑、灯光开关 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-viper-v2-pro | [Razer Viper V2 Pro](../drivers/razer-viper-v2-pro-wired) | 1532:00A5 | 电量、设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-wired-settings | [Razer DeathAdder Essential](../drivers/razer-deathadder-essential) | 1532:006E | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-wired-settings | [Razer DeathAdder V2](../drivers/razer-deathadder-v2) | 1532:0084 | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-wired-settings | [Razer Mamba Elite](../drivers/razer-mamba-elite) | 1532:006C | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-wired-settings | [Razer Naga Trinity](../drivers/razer-naga-trinity) | 1532:0067 | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-wired-settings | [Razer Viper 8KHz](../drivers/razer-viper-8khz) | 1532:0091 | 设置 | 预览 | 接口匹配 | 公开实现 |
| protocol-razer-wireless | [Razer Atheris](../drivers/razer-atheris) | 1532:0062 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Basilisk Mobile](../drivers/razer-basilisk-mobile/PROVENANCE.md) | 1532:00D4 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Basilisk Ultimate](../drivers/razer-basilisk-ultimate) | 1532:0088 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Basilisk V3 Pro](../drivers/razer-basilisk-v3-pro/PROVENANCE.md) | 1532:00AB | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Basilisk V3 Pro 35K](../drivers/razer-basilisk-v3-pro-35k/PROVENANCE.md) | 1532:00CD | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Basilisk V3 Pro 35K Phantom](../drivers/razer-basilisk-v3-pro-35k-phantom/PROVENANCE.md) | 1532:00D7 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Basilisk V3 X HyperSpeed](../drivers/razer-basilisk-v3-x-hyperspeed/PROVENANCE.md) | 1532:00B9 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Basilisk X HyperSpeed](../drivers/razer-basilisk-x-hyperspeed) | 1532:0083 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Cobra HyperSpeed](../drivers/razer-cobra-hyperspeed/PROVENANCE.md) | 1532:00DB | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Cobra Pro](../drivers/razer-cobra-pro/PROVENANCE.md) | 1532:00B0 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer DeathAdder V2 Pro 无线接收器](../drivers/razer-deathadder-v2-pro-receiver) | 1532:007D | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer DeathAdder V2 X HyperSpeed](../drivers/razer-deathadder-v2-x-hyperspeed) | 1532:009C | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer DeathAdder V4 Pro](../drivers/razer-deathadder-v4-pro/PROVENANCE.md) | 1532:00BF | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Lancehead](../drivers/razer-lancehead/PROVENANCE.md) | 1532:005A | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Lancehead Wireless 无线接收器](../drivers/razer-lancehead-wireless-receiver) | 1532:006F | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Mamba Chroma](../drivers/razer-mamba-chroma/PROVENANCE.md) | 1532:0045 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Mamba Wireless 无线接收器](../drivers/razer-mamba-wireless-receiver) | 1532:0072 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Naga Pro](../drivers/razer-naga-pro/UPSTREAM.md) | 1532:0090 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Naga V2 HyperSpeed](../drivers/razer-naga-v2-hyperspeed/PROVENANCE.md) | 1532:00B4 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Naga V2 Pro](../drivers/razer-naga-v2-pro) | 1532:00A8 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Orochi V2](../drivers/razer-orochi-v2) | 1532:0094 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Pro Click](../drivers/razer-pro-click) | 1532:0077 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Pro Click Mini](../drivers/razer-pro-click-mini/PROVENANCE.md) | 1532:009A | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Pro Click V2](../drivers/razer-pro-click-v2/PROVENANCE.md) | 1532:00D1 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Pro Click V2 Vertical](../drivers/razer-pro-click-v2-vertical/PROVENANCE.md) | 1532:00C8 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Viper Mini Signature Edition](../drivers/razer-viper-mini-signature-edition/UPSTREAM.md) | 1532:009F | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Viper Ultimate](../drivers/razer-viper-ultimate-receiver/UPSTREAM.md) | 1532:007B | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Viper V2 Pro 无线接收器](../drivers/razer-viper-v2-pro-receiver) | 1532:00A6 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-razer-wireless | [Razer Viper V3 HyperSpeed](../drivers/razer-viper-v3-hyperspeed/PROVENANCE.md) | 1532:00B8 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-razer-wireless | [Razer Viper V3 Pro](../drivers/razer-viper-v3-pro/PROVENANCE.md) | 1532:00C1 | 电量 | 预览 | 接口匹配 | 上游实机 |
| razer-deathadder-v3-hyperspeed | [Razer DeathAdder V3 HyperSpeed](../drivers/razer-deathadder-v3-hyperspeed/PROVENANCE.md) | 1532:00C5 | 电量、读 DPI 档位配置、编辑 DPI 档位、读配置回报率、设置回报率、指针设置 | 预览 | 接口匹配 | 上游实机 |
| razer-deathadder-v3-pro-alt | [Razer DeathAdder V3 Pro (alternate receiver)](../drivers/razer-deathadder-v3-pro-alt/PROVENANCE.md) | 1532:00C3 | 电量、读 DPI 档位配置、编辑 DPI 档位、读配置回报率、设置回报率、指针设置 | 预览 | 接口匹配 | 上游实机 |
| razer-viper-v3-pro-se | [Razer Viper V3 Pro SE](../drivers/razer-viper-v3-pro-se/PROVENANCE.md) | 1532:00DF | 电量、设置 | 预览 | 接口匹配 | 上游实机 |
| razer-viper-v4-pro | [Razer Viper V4 Pro](../drivers/razer-viper-v4-pro/PROVENANCE.md) | 1532:00E6 1532:00E5 | 电量、设置 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-royal-kludge"></a>

## ROYAL KLUDGE

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| royal-kludge-m3 | [Royal Kludge M3](../drivers/royal-kludge-m3/UPSTREAM.md) | 372E:1019 372E:102A | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-scyrox"></a>

## SCYROX

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-scyrox | [Scyrox V6](../drivers/scyrox-v6/UPSTREAM.md) | 3554:F5F6 3554:F5F7 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-scyrox | [Scyrox V8S](../drivers/scyrox-v8s/UPSTREAM.md) | 3554:F515 3554:F517 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-sony"></a>

## Sony

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-dualshock4 | [Sony DualShock 4](../drivers/sony-dualshock-4) | 054C:09CC | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-sony-inzone-h9-ii | [Sony INZONE H5](../drivers/sony-inzone-h5) | 054C:0EBF | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-sony-inzone-h9-ii | [Sony INZONE H9 II (WH-G900N2)](../drivers/sony-inzone-h9-ii/README.md) | 054C:0FA8 | 电量 | 预览 | 接口匹配 | 上游实机 |
| sony-dualsense-bluetooth | [Sony DualSense Bluetooth](../drivers/sony-dualsense-bluetooth/UPSTREAM.md) | 054C:0CE6 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-steam"></a>

## STEAM

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-steam-puck | [Steam Controller 2025](../drivers/steam-controller-2025) | 28DE:1304 28DE:1305 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-steelseries"></a>

## SteelSeries

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-steelseries-aerox-9-wireless | [SteelSeries Aerox 9 Wireless](../drivers/steelseries-aerox-9-wireless/UPSTREAM.md) | 1038:1858 1038:1874 1038:185A 1038:1876 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-steelseries-arctis | [SteelSeries Arctis Nova 3P/3X Wireless](../drivers/steelseries-arctis-nova-3p-wireless) | 1038:2269 1038:226D | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-steelseries-arctis | [SteelSeries Arctis Nova 7 / 5 family](../drivers/steelseries-arctis-nova-7) | 1038:22A1 1038:2202 1038:227E 1038:2206 1038:2258 1038:229E 1038:22AD 1038:22A4 1038:22A5 1038:223A 1038:22A9 1038:227A 1038:220A 1038:22A7 1038:2298 1038:2232 1038:2253 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis | [SteelSeries Arctis Nova Pro Wireless](../drivers/steelseries-arctis-nova-pro-wireless) | 1038:12E0 1038:12E5 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis-legacy | [SteelSeries Arctis 1 Wireless](../drivers/steelseries-arctis-1) | 1038:12B3 1038:12B6 | 电量 | 预览 | 接口匹配 | 上游验证 |
| protocol-steelseries-arctis-legacy | [SteelSeries Arctis 7 (2019)](../drivers/steelseries-arctis-7-2019/UPSTREAM.md) | 1038:12AD | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis-legacy | [SteelSeries Arctis 7+](../drivers/steelseries-arctis-7-plus/UPSTREAM.md) | 1038:220E 1038:2212 1038:2216 1038:2236 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis-legacy | [SteelSeries Arctis 9](../drivers/steelseries-arctis-9/UPSTREAM.md) | 1038:12C2 1038:12C4 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis-legacy | [SteelSeries Arctis Pro Wireless](../drivers/steelseries-arctis-pro-wireless/UPSTREAM.md) | 1038:1290 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis-legacy | [SteelSeries Arctis GameBuds](../drivers/steelseries-gamebuds/UPSTREAM.md) | 1038:230A | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-arctis-nova-elite | [SteelSeries Arctis Nova Elite](../drivers/steelseries-arctis-nova-elite/UPSTREAM.md) | 1038:2244 | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-steelseries-mice | [SteelSeries Aerox 3 Wireless](../drivers/steelseries-aerox-3-wireless) | 1038:1838 1038:1878 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-mice | [SteelSeries Aerox 5 Wireless](../drivers/steelseries-aerox-5-wireless) | 1038:1852 1038:185C 1038:1860 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-steelseries-mice | [SteelSeries Rival 3 Wireless](../drivers/steelseries-rival-3-wireless) | 1038:1830 1038:1872 | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-thunderobot"></a>

## THUNDEROBOT

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| thunderobot-ml903-nearlink-se | [THUNDEROBOT ML903 NearLink Se](../drivers/thunderobot-ml903-nearlink-se/PROVENANCE.md) | 369B:0234 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-vgn"></a>

## VGN

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-vgn | [VGN F1 Pro](../drivers/vgn-f1-pro) | 3554:F503 3554:F502 | 电量 | 预览 | 接口匹配 | 公开实现 |
| protocol-vgn | [VGN Dragonfly F2 Master+](../drivers/vgn-f2-master-plus) | 3554:FB56 3554:FB57 | 电量、设置、改键 | 预览 | 接口匹配 | 公开实现 |
| protocol-vgn | [VGN Dragonfly F2 Pro Max](../drivers/vgn-f2-pro-max) | 3554:FB3E 3554:FB3D | 电量 | 预览 | 接口匹配 | 公开实现 |
| vgn-f1-v5-turbo-plus | [VGN F1 V5 Turbo+](../drivers/vgn-f1-v5-turbo-plus/PROVENANCE.md) | 391D:1A23 391D:1026 | 电量 | 预览 | 接口匹配 | 上游实机 |
| vgn-v98-pro | [VGN V98 Pro](../drivers/vgn-v98-pro/UPSTREAM.md) | 320F:5088 | 电量 | 预览 | 接口匹配 | 上游实机 |

<a id="vendor-vxe"></a>

## VXE

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-pulsar-17 | [VXE R1 Pro Max](../drivers/vxe-r1-pro-max/UPSTREAM.md) | 3554:F58A 3554:F58C | 电量 | 预览 | 接口匹配 | 上游实机 |
| protocol-pulsar-17 | [VXE R1 SE+](../drivers/vxe-r1-se-plus/UPSTREAM.md) | 373B:1085 3554:F58F | 电量 | 预览 | 接口匹配 | 代码适配 |

<a id="vendor-wlmouse"></a>

## WLMOUSE

| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |
| --- | --- | --- | --- | --- | --- | --- |
| protocol-wlmouse | [WLmouse Beast X](../drivers/wlmouse-beast-x/UPSTREAM.md) | 36A7:A887 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-wlmouse | [WLmouse Beast X Max](../drivers/wlmouse-beast-x-max/UPSTREAM.md) | 36A7:A880 | 电量 | 预览 | 接口匹配 | 代码适配 |
| protocol-wlmouse | [WLmouse Beast X Mini Pro](../drivers/wlmouse-beast-x-mini-pro/UPSTREAM.md) | 36A7:A868 | 电量 | 预览 | 接口匹配 | 代码适配 |
