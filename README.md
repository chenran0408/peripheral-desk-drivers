# 外设盒设备插件

外设盒（原 Peripheral Desk）的公开设备驱动包仓库。每款设备使用独立 DLL，可在主程序的驱动商店单独安装和更新。

| 驱动包 | 设备 |
| --- | --- |
| `razer-deathadder-v3-pro` | 雷蛇炼狱蝰蛇 V3 专业版 |
| `mchose-a5-ultra` | 迈从 A5 Ultra 初代 |
| `mchose-g75-pro` | 迈从 G75 Pro |
| `hecate-g2-pro` | 漫步者 G2 Pro |
| `logitech-g305`（预览） | G305 LIGHTSPEED，电量、条件允许时的 DPI 设置；需关闭板载配置，尚未实机验证 |
| `razer-basilisk-v3-pro-wired`（预览） | Basilisk V3 Pro 有线，电量、DPI、回报率、灯效；尚未实机验证 |
| `mchose-a7-v3-ultra-plus`（预览） | A7 V3 Ultra+ 接收器，电量、DPI、回报率、鼠标键映射；尚未实机验证 |
| `vgn-f2-master-plus`（预览） | F2 Master+ 有线/接收器，电量、DPI、档位、回报率、鼠标键映射；尚未实机验证 |
| `atk-x1-pro-max`（预览） | X1 Pro Max 有线/8K 接收器，电量、DPI、回报率；尚未实机验证 |
| `razer-viper-v2-pro-wired`（预览） | Viper V2 Pro 有线，电量、DPI、回报率，尚未实机验证 |
| `atk-a9-plus-nearlink`（预览） | A9 Plus Nearlink 有线/NearLink 接收器，电量、DPI、最高 1000 Hz 回报率；尚未实机验证 |
| `razer-viper-ultimate-wired`（预览） | Viper Ultimate 有线，电量、DPI、回报率、Logo 灯效；尚未实机验证 |
| `vgn-f2-pro-max`（预览） | F2 Pro Max 有线/接收器，电量与充电状态 |
| `vgn-f1-pro`（预览） | F1 Pro 有线/接收器，电量与充电状态 |
| `razer-deathadder-v2-pro-wired`（预览） | DeathAdder V2 Pro 有线，电量、DPI、回报率、Logo 灯效；尚未实机验证 |

| `ajazz-aj179-pro`（预览） | AJ179 PRO USB / 接收器，DPI、回报率、休眠、鼠标键映射；上游已实测，本项目未复测 |
| `mchose-magdock`（预览） | MagDock 独立底座，灯光开关、模式、颜色、亮度、速度；主程序需 0.4.8 |
| `mchose-v9-pro`（预览） | MCHOSE V9 Pro 2.4 GHz 接收器，只读电量；主动查询，充电状态未知，非满电映射尚无证据 |
| `mchose-g87-v1`（预览） | MCHOSE G87 V1，接收器电量与充电状态；不支持 G87 V2/QHW |
| `mchose-g87-v2`（预览） | MCHOSE G87 V2 精确接收器，只读电量/充电（独立代际插件） |
| `mchose-a7-v2-pro`（预览） | MCHOSE A7 V2 Pro 接收器被动状态与电量；与 A7 V3 分开 |
| `akg-n9-hybrid`（预览） | AKG N9 Hybrid 接收器，只读电量 |
| `asus-rog-scope-ii-96-omni`（预览） | ROG Scope II 96 Omni 配对接收器，只读电量 |
| `atk-a98-pro`（预览） | ATK A98 Pro VIA 电量；没有型号专属写入定义 |
| `atk-f1-pro-max`（预览） | ATK F1 Pro Max 电量、DPI 与回报率；有线/普通接收器至 1 kHz，8K 接收器至 8 kHz |
| `atk-f1-v3-ultimate-plus`（预览） | ATK F1 V3 ULTIMATE+ 指定 8K-M 接收器/有线只读电量；有线报告长度未上游实测 |
| `atk-f1-ultimate-2-0`（预览） | ATK F1 Ultimate 2.0 指定 VID/PID 只读电量；型号协议尚无实机报告 |
| `aula-f108-pro`（预览） | AULA F108 Pro 接收器只读电量 |
| `aula-hero-68-air`（预览） | AULA HERO 68 Air 接收器电量与充电状态，需 UUID 匹配 |
| `haute42-adapter-4ds`（预览） | Haute42 Adapter 4DS 只读识别与电量；普通 DualSense 不适用 |
| `hyperx-cloud-flight-2`（预览） | HyperX Cloud Flight 2 被动读取电量、充电与连接状态 |
| `logitech-pro-x2-lightspeed`（预览） | Logitech PRO X 2 LIGHTSPEED 接收器只读电量；不含 EQ/环绕声 |
| `razer-blackshark-v2-pro-2023`（预览） | 2023 版接收器电量/充电主动查询；需在设备页启用 |
| `razer-kraken-v4`（预览） | Razer Kraken V4 接收器只读电量/充电 |
| `razer-viper-v3-pro-se`（预览） | Viper V3 Pro SE 接收器电量、DPI 与最高 1 kHz 回报率 |
| `razer-viper-v4-pro`（预览） | Viper V4 Pro 电量、DPI、回报率与休眠设置 |
| `sony-inzone-h9-ii`（预览） | Sony INZONE H9 II 接收器电量与充电状态 |
| `vgn-f1-v5-turbo-plus`（预览） | VGN F1 V5 Turbo+ 有线/接收器只读电量与充电状态 |
| `vgn-v98-pro`（预览） | VGN V98 Pro 接收器只读电量与充电状态 |
| `aula-g9`（预览） | AULA G9 被动读取电量/充电；没有音效或灯光控制 |
| `aula-l9-ultra`（预览） | AULA L9 Ultra 接收器电量与充电状态 |
| `betop-t9`（预览） | BETOP T9 被动接收器电量 |
| `betop-kunpeng20`（预览） | BETOP 鲲鹏20 2.4G/有线电量与已知充电状态；不含 5191 变体 |
| `cherry-pixiu75`（预览） | CHERRY PIXIU75 接收器/有线电量与充电状态 |
| `delux-m900-pro`（预览） | DELUX M900 Pro 独立被动报告接口，只读电量与充电 |
| `eweadn-s9-series`（预览） | EWEADN S9 Pro/Max/Ultra 系列接收器电量与充电 |
| `eweadn-x87`（预览） | EWEADN X87 接收器只读电量 |
| `inphic-zhen`（预览） | INPHIC ZHEN 被动电量/充电，和共用 VID/PID 的 DELUX 分接口识别 |
| `lingbao-m5pro-max`（预览） | 凌豹 M5PRO Max 接收器电量与充电；排除仅回显的有线通道 |
| `rapoo-vt3-ii`（预览） | Rapoo VT3 II 被动心跳电量/充电 |
| `ajazz-ak870`（预览） | AJAZZ AK870 接收器电量，只读 |
| `thunderobot-ml903-nearlink-se`（预览） | 雷神 ML903 NearLink Se 接收器只读电量 |
| `sony-dualsense-bluetooth`（预览） | DualSense 蓝牙只读 10% 档位电量；不适用 USB/Edge/Haute42 |
| `pulsar-x2-v2`（预览） | Pulsar X2 V2 原装 1 kHz 接收器电量/充电；休眠时电量可未知 |
| `razer-darkrim-shadow`（预览） | DARKRIM SHADOW 精确接收器电量；官方程序运行或进程状态不明时让位 |
| `gamesir-g7-pro-ce`（预览） | GameSir G7 Pro CE 手柄电量/充电；按设备请求主动发送只读心跳 |
| `hecate-g5m-pro`（预览） | HECATE G5M Pro 接收器电量/充电；主动只读查询 |
| `hecate-g3m-pro-upgrade`（预览） | HECATE D3FB 接收器电量；接收器协议不确认配对鼠标型号 |
| `logitech-bolt-mx-master-3s`（预览） | Logitech Bolt 配对 MX Master 3S 电量/充电；先核对精确配对设备名 |

Vigil Watt 的经典蓝牙 EDIFIER W820NB 电量读取来自 Windows HFP 设备节点属性，不经过 HID。现有插件发现接口无法枚举此类系统设备，因此这款暂未提供插件；兼容清单记录了所需的主程序侧系统电池接入。

## 下载

在 [Peripheral Desk](https://github.com/chenran0408/peripheral-desk/releases) 下载主程序，再到驱动商店选择设备。手动下载可用本仓库的 [发布页面](https://github.com/chenran0408/peripheral-desk-drivers/releases)。每个 ZIP 对应一款设备；请通过主程序安装，不要双击或注册 DLL。离线安装需同时保留来自维护者同一 release 的受信 catalog-stable.json（或预览通道目录）及其 `.json.sig`，由主程序核对签名、ZIP 与 DLL 的 SHA256。

G2 Pro 插件 0.4.10 附带所需 x64 SDK、官方音频驱动、扩展和 APO，自动连接音效组件；首次使用可在 EQ 页点击“安装音效驱动”，无需运行官方控制面板。主程序仍不预装任何设备插件。厂商组件的版权和来源单独列于 [组件说明](docs/G2-PRO-COMPONENTS.md)。

A5 Ultra 的板载宏仍为实验功能：当前实测固件未返回可完整备份的宏页，因此保持拒写。电量、设置与六键改键可分别使用；不能把宏编辑窗口可打开当作设备宏已验证可用。

## 开发

需要 Windows x64、Rust 和 Python 3.11+。设备插件共用 ABI 1；现有四款设备使用固定版本的 Peripheral Desk 核心库，新型号可在本仓库提供独立协议模块。只要 ABI 与请求模型保持兼容，新增设备可以在本仓库完成，不必改主界面的设备列表。

- [本地构建与贡献](CONTRIBUTING.md)
- [社区设备适配 Wiki（含可交给 Agent 的提示词）](docs/COMMUNITY-ADAPTATION-WIKI.md)
- [上游实现与证据复用](docs/UPSTREAM-REUSE.md)
- [后续接入任务与验收标准](docs/ADAPTATION-ROADMAP.md)
- [设备目录：品牌—协议系列—型号—连接方式](docs/DEVICE-CATALOG.md)
- [C ABI 与 JSON 请求](docs/ABI.md)
- [包和目录格式](docs/PACKAGES.md)
- [新设备模板](templates/device/README.md)
- [A7 V3 Ultra+ 接收器预览](docs/MCHOSE-A7-V3.md)
- [VGN F2 Master+ 预览](docs/VGN-F2-MASTER-PLUS.md)
- [图片来源](ASSET_SOURCES.md)
- [G2 Pro 灯光接口与实测边界](docs/G2_LIGHTING.md)
- [五品牌协议调查、固定来源版本与许可证](docs/PROTOCOL-RESEARCH.md)
- [Vigil Watt v1.3.0 全设备迁移清单、上游验证范围与未接入项](docs/VIGIL-WATT-MIGRATION.md)
- [G305 只读预览的精确型号与连接范围](docs/LOGITECH-G305.md)
- [Basilisk V3 Pro 有线预览的协议依据与范围](docs/RAZER-BASILISK-V3-PRO.md)

代码采用 AGPL-3.0-only，见 [LICENSE](LICENSE)。项目与设备品牌无隶属关系。

支持 ABI 1、主程序 0.4.x；提供 stable / preview 两个通道。每次更新可查看简短改进说明；详见 [兼容表](docs/COMPATIBILITY.md)。

商店分别读取固定的 [稳定版目录](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/catalog-stable/catalog-stable.json) 和 [预览版目录](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/catalog-preview/catalog-preview.json)。预览发布不会替换稳定版目录。

## 目录签名与离线安装

主程序只安装维护者审核并签名的 release 目录所列包。catalog JSON 及同名 `.json.sig` 必须来自同一次受信发布；主程序先使用内置 RSA 公钥验证 SHA256/PKCS1 签名，再核对 ZIP 和 DLL hash。离线安装也必须携带该签名目录与 `.sig`，任意 JSON 或 PR 的未签名构建产物均不能作为可信安装来源。包 ZIP 自身不循环包含目录签名。

公钥为 `assets/catalog-public-key.blob`（BCRYPT_RSAPUBLIC_BLOB），可公开。私钥只放本地私密目录或 GitHub Secret `DRIVER_CATALOG_SIGNING_KEY`，绝不进入源码、包或日志。本地维护者用 `scripts/sign-catalog.ps1 -PrivateKeyPath <私钥XML路径>`；也可从同名环境变量读取 XML。`scripts/verify-signatures.ps1` 只需要公钥。

PR 与普通主分支 CI 只构建、核对 ABI 和包格式，不签名、不发布。受审的 main 提交上的现有 tag，或从 main 手动选择同一 tag，才会使用 Secret 签名并发布；发布后 CI 复核公开目录签名与所有包。签名不等于安全审计，维护者仍须检查设备匹配、协议和失败恢复。
