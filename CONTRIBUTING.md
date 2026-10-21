# 添加设备与更新驱动

## 本地构建

辅助脚本使用 Python 3.11 或更新版本（CI 使用 3.12）。[接入任务队列](docs/ADAPTATION-ROADMAP.md)按证据和可复用协议安排新型号，领取任务前先核对现有实现。

发布树固定核心库的 Git revision；开发时可临时使用相邻的 `../PeripheralDesk`，需要它提供本设备使用的驱动入口与 feature。目录可安排为：

```text
outputs/
  PeripheralDesk/
  PeripheralDeskDrivers/
```

在 Windows PowerShell 中，从本仓库运行：

```powershell
$env:CARGO_TARGET_DIR = "D:\your-task\work\driver-target"
python scripts/build.py --target x86_64-pc-windows-msvc
```

如使用 GNU 工具链，改为 `--target x86_64-pc-windows-gnu`。必要时通过 `--cargo` 指定 Cargo 路径、`--toolchain` 指定已安装工具链。产物在 `dist/`：每设备一个 ZIP、`catalog.json` 和 ZIP 校验值；目录中的每个条目还包含 DLL 的 SHA256。`--skip-build` 仅重新打包已构建的 DLL。

构建不启动 DLL，也不向实物发送任何命令。不要在公开包中包含用户 `data/`、硬件抓包、机器路径或厂商二进制。

## 发布前固定核心版本

开发期的 `path` 依赖不能直接用于公开发布。先把核心更改提交并公开，然后把根 Cargo.toml 的依赖改成：

```toml
peripheral-desk = { git = "https://github.com/chenran0408/peripheral-desk", rev = "<完整40位提交SHA>", default-features = false, features = ["driver-runtime"] }
```

更新并提交 Cargo.lock，再运行 `python scripts/build.py --release --target <已安装的Windows-x64目标>`。`--release` 会拒绝 path 依赖或未固定的 Git revision，并使用锁文件构建。每款驱动可独立升级：修改对应 crate 的版本和所选 metadata 中该设备的 version，不要求其他设备或主程序跟着升级。继承 workspace version 的 crate 仍需确保其实际版本与 metadata 一致。

发布到其他标签时增加 `--release-tag v0.4.1-preview.1`。该参数是**实际接收本次所有 ZIP 的 release 标签**，可以和单个包的版本不同；CI 会把触发标签传入，再从已发布文件验证目录下载地址。省略时使用 `v<workspace version>`。stable 和 preview 的包都指向这次不可变版本发布中的 ZIP；通道目录地址另外固定，不依赖 GitHub 的 latest。

只发布一款稳定驱动，例如：`python scripts/build.py --release --metadata drivers.json --package mchose-a5-ultra --release-tag v0.4.1 --out dist`。只发布 G305 预览版则用 `--metadata drivers-preview.json --package logitech-g305 --release-tag v0.4.1-preview.1`。重复 `--package` 可选多款；打包分别构建设备，避免依赖功能合并把无关协议带入每个 DLL。不要为改内容复用同一设备版本；同版不同哈希和版本倒退会被频道发布器拒绝。

## 新设备步骤

1. 从 `templates/device` 复制一个包，替换 id、crate 名和 metadata。四款已有包从0.4.1起使用各自的专用协议入口；新协议使用模板的自定义 handler，保持同样的 JSON 响应格式，避免通过全设备 dispatcher 牵入无关功能。
2. 把成员加入 workspace，放入清晰且有来源的设备图片，按通道更新 drivers.json 或 drivers-preview.json。匹配规则必须包含明确的 VID/PID，并按实证补充 usage、report 长度或产品名，避免匹配整个品牌。
3. 先实现只读状态，再实现已确认的设置。保留未知字段、隐藏槽和其他按键；失败时恢复可完整备份的原设置。
4. 建立少量有意义的 codec 测试。硬件写入验证须另有授权；说明已实测、仅依据资料实现和暂未支持的边界。
5. 在 PR 中写明设备及连接方式、协议第一手来源、用户可见变化、恢复行为和验证结果。不要夹带固件更新或系统驱动卸载。

## PR 检视重点

- ABI 1 三导出与 Buffer 所有权是否完全一致，是否捕获 Rust unwind。
- JSON 字段是否与宿主兼容，未知操作是否返回明确错误。
- metadata 是否会误识别其他型号；catalog 与包 manifest 的 device 是否完全相同。
- 是否验证范围、避免重复写命令，并确认读回；无法读回的功能是否明确说明。
- DLL 和 ZIP 是否各计算 SHA256，下载地址是否指向对应版本。
- 是否只修改本设备，能否在不覆盖未知数据的前提下恢复；外部组件是否由用户自行安装。
- 贡献的代码和图片是否有可说明的来源，许可证是否保留。

CI 在 Windows x64 构建所选包，核对 ABI、包清单、兼容范围及 SHA256，并上传可下载构建产物；release-check 另外检查维护者已发布的包。稳定 tag 默认选 drivers.json，带连字符的预览 tag 默认选 drivers-preview.json；手动运行可选 metadata 和逗号分隔的设备包名。只有受审 main tag / 手动发布流程可签名并发布，任何 PR 均不会创建发布。版本发布只上传本次所选 ZIP、subset catalogs、签名及 SHA256SUMS.txt。

先运行 `python scripts/check-source.py` 检查元数据、模板与本地文档链接。它也核对每款设备的 Cargo 包名、实际版本（含继承 workspace 的版本）与目录一致，并确认成员已加入 workspace；新增或单独升级设备时不能只改目录版本。ABI 原生探测在限时 30 秒的独立 Python 子进程中进行，异常 DLL 不会无限卡住构建。CI 会验证上传后的真实下载文件；由 GITHUB_TOKEN 创建 release 不会触发另一条 release workflow，因此发布任务自身也执行这一步。

版本发布一经存在就拒绝覆盖，请修正后使用新标签。上传先创建 draft，全部成功后才公开；若中途失败，请检查残留 draft 后重试。发布任务串行更新 `catalog-stable` / `catalog-preview` 两个预发布页上的目录及签名，这两个页面只存目录，不放设备 ZIP。目录替换时客户端如读到暂时不匹配的一对文件会保留上次已验证目录。空通道仅首发初始化，之后不会用空目录覆盖已有内容。

不可变版本发布中的目录只列本批所选设备，便于与同一 release 的 ZIP 一一验证。频道发布器先下载并验证已有通道目录及签名，按设备 id 合并更新，完整保留其他设备条目，再重新签名后发布完整通道目录；不会改动版本发布的 subset catalogs。空输入保留另一个通道。删除设备须另行明确维护，不用遗漏条目或空目录暗示删除，也不能复用旧签名覆盖合并后的 JSON。

可用 `python scripts/verify.py dist/catalog-stable.json --abi-check` 在64位Windows检查 ABI；该检查只传入无效UTF-8，在协议派发前即返回，不访问设备。驱动 manifest 的 core_min/core_max_exclusive、channel、capabilities 和 changelog 必须与 catalog 一致。

## 目录签名与离线安装

主程序只安装维护者审核并签名的 release 目录所列包。catalog JSON 及同名 `.json.sig` 必须来自同一次受信发布；主程序先使用内置 RSA 公钥验证 SHA256/PKCS1 签名，再核对 ZIP 和 DLL hash。离线安装也必须携带该签名目录与 `.sig`，任意 JSON 或 PR 的未签名构建产物均不能作为可信安装来源。包 ZIP 自身不循环包含目录签名。

公钥为 `assets/catalog-public-key.blob`（BCRYPT_RSAPUBLIC_BLOB），可公开。私钥只放本地私密目录或 GitHub Secret `DRIVER_CATALOG_SIGNING_KEY`，绝不进入源码、包或日志。本地维护者用 `scripts/sign-catalog.ps1 -PrivateKeyPath <私钥XML路径>`；也可从同名环境变量读取 XML。`scripts/verify-signatures.ps1` 只需要公钥。

PR 与普通主分支 CI 只构建、核对 ABI 和包格式，不签名、不发布。受审的 main 提交上的现有 tag，或从 main 手动选择同一 tag，才会使用 Secret 签名并发布；发布后 CI 复核公开目录签名与所有包。签名不等于安全审计，维护者仍须检查设备匹配、协议和失败恢复。

仓库维护者须配置 Actions Secret `DRIVER_CATALOG_SIGNING_KEY`（完整 RSA 私钥 XML）。公钥文件必须与主程序内置公钥一致；只有该签名密钥的维护者能使目录成为可安装来源。`scripts/publish-channels.ps1 -Repository chenran0408/peripheral-desk-drivers` 可用于本地维护者发布通道，调用前必须保证没有其他发布者并行运行。
