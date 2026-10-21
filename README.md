# Peripheral Desk Drivers

Peripheral Desk 的公开设备驱动包仓库。每款设备使用独立 DLL，可在主程序的驱动商店单独安装和更新。

| 驱动包 | 设备 |
| --- | --- |
| `razer-deathadder-v3-pro` | 雷蛇炼狱蝰蛇 V3 专业版 |
| `mchose-a5-ultra` | 迈从 A5 Ultra 初代 |
| `mchose-g75-pro` | 迈从 G75 Pro |
| `hecate-g2-pro` | 漫步者 G2 Pro |
| `logitech-g305`（预览） | G305 LIGHTSPEED，只读电量，依据公开实现适配、尚未实机验证 |
| `razer-basilisk-v3-pro-wired`（预览） | Basilisk V3 Pro 有线 1532:00AA，只读电量与充电状态，尚未实机验证 |
| `mchose-a7-v3-ultra-plus`（预览） | A7 V3 Ultra+ 3837:1018 接收器，只读电量，尚未实机验证 |
| `vgn-f2-master-plus`（预览） | F2 Master+ 3554:FB56/FB57，只读电量与充电状态，尚未实机验证 |
| `atk-x1-pro-max`（预览） | X1 Pro Max 有线与 8K 接收器，只读电量、运行时型号校验，尚未实机验证 |
| `razer-viper-v2-pro-wired`（预览） | Viper V2 Pro 有线，只读电量与充电状态，尚未实机验证 |
| `atk-a9-plus-nearlink`（预览） | A9 Plus Nearlink 有线与 NearLink 接收器，只读电量，尚未实机验证 |
| `razer-viper-ultimate-wired`（预览） | Viper Ultimate 有线，只读电量与充电状态，尚未实机验证 |
| `vgn-f2-pro-max`（预览） | F2 Pro Max 有线/接收器，电量与充电状态 |
| `vgn-f1-pro`（预览） | F1 Pro 有线/接收器，电量与充电状态 |
| `razer-deathadder-v2-pro-wired`（预览） | DeathAdder V2 Pro 有线，电量与充电状态 |

## 下载

在 [Peripheral Desk](https://github.com/chenran0408/peripheral-desk/releases) 下载主程序，再到驱动商店选择设备。手动下载可用本仓库的 [发布页面](https://github.com/chenran0408/peripheral-desk-drivers/releases)。每个 ZIP 对应一款设备；请通过主程序安装，不要双击或注册 DLL。离线安装需同时保留来自维护者同一 release 的受信 catalog-stable.json（或预览通道目录）及其 `.json.sig`，由主程序核对签名、ZIP 与 DLL 的 SHA256。

G2 Pro 的音效继续使用已安装的 HECATE / C-Media 组件，驱动包不附带厂商安装包或 DLL。支持范围以商店说明为准。

A5 Ultra 的板载宏仍为实验功能：当前实测固件未返回可完整备份的宏页，因此保持拒写。电量、设置与六键改键可分别使用；不能把宏编辑窗口可打开当作设备宏已验证可用。

## 开发

需要 Windows x64、Rust 和 Python 3.11+。设备插件共用 ABI 1；现有四款设备使用固定版本的 Peripheral Desk 核心库，新型号可在本仓库提供独立协议模块。只要 ABI 与请求模型保持兼容，新增设备可以在本仓库完成，不必改主界面的设备列表。

- [本地构建与贡献](CONTRIBUTING.md)
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
- [G305 只读预览的精确型号与连接范围](docs/LOGITECH-G305.md)
- [Basilisk V3 Pro 有线预览的协议依据与范围](docs/RAZER-BASILISK-V3-PRO.md)

代码采用 AGPL-3.0-only，见 [LICENSE](LICENSE)。项目与设备品牌无隶属关系。

支持 ABI 1、主程序 0.4.x；提供 stable / preview 两个通道。每次更新可查看简短改进说明；详见 [兼容表](docs/COMPATIBILITY.md)。

商店分别读取固定的 [稳定版目录](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/catalog-stable/catalog-stable.json) 和 [预览版目录](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/catalog-preview/catalog-preview.json)。预览发布不会替换稳定版目录。

## 目录签名与离线安装

主程序只安装维护者审核并签名的 release 目录所列包。catalog JSON 及同名 `.json.sig` 必须来自同一次受信发布；主程序先使用内置 RSA 公钥验证 SHA256/PKCS1 签名，再核对 ZIP 和 DLL hash。离线安装也必须携带该签名目录与 `.sig`，任意 JSON 或 PR 的未签名构建产物均不能作为可信安装来源。包 ZIP 自身不循环包含目录签名。

公钥为 `assets/catalog-public-key.blob`（BCRYPT_RSAPUBLIC_BLOB），可公开。私钥只放本地私密目录或 GitHub Secret `DRIVER_CATALOG_SIGNING_KEY`，绝不进入源码、包或日志。本地维护者用 `scripts/sign-catalog.ps1 -PrivateKeyPath <私钥XML路径>`；也可从同名环境变量读取 XML。`scripts/verify-signatures.ps1` 只需要公钥。

PR 与普通主分支 CI 只构建、核对 ABI 和包格式，不签名、不发布。受审的 main 提交上的现有 tag，或从 main 手动选择同一 tag，才会使用 Secret 签名并发布；发布后 CI 复核公开目录签名与所有包。签名不等于安全审计，维护者仍须检查设备匹配、协议和失败恢复。
