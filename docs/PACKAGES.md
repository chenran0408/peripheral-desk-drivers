# 包、清单与商店目录

每个 ZIP 的根目录包含 `driver.dll`、`device.png`、`manifest.json`、`LICENSE`、`README.txt`，可附带依赖许可证。不要设置嵌套的顶层包目录。metadata.image 固定为 device.png，DLL 名固定为 driver.dll。

包 manifest：

```json
{
  "schema": 1, "abi": 1,
  "id": "example-device", "name": "示例设备", "version": "0.4.0",
  "description": "给用户看的简短功能说明",
  "dll": "driver.dll", "dll_sha256": "64位十六进制SHA256",
  "core_min": "0.4.0", "core_max_exclusive": "0.5.0",
  "channel": "preview", "capabilities": ["settings"],
  "changelog": "首次预览，开放已确认设置。",
  "device": {
    "key": "example-device", "name": "示例设备", "brand": "EXAMPLE",
    "kind": "mouse", "subtitle": "", "image": "device.png",
    "matches": [{"vid": 4660, "pid": 22136, "usage_page": 65281, "feature_len": 65}],
    "advanced": []
  }
}
```

公开 catalog 格式为 `{"schema":1,"channel":"stable","drivers":[...]}`，preview 使用相同结构。每个条目有 manifest 的 id/name/version/description/abi/dll/dll_sha256/device，另外包含 `download_bytes`、`package_url`、`sha256`。sha256 为整个 ZIP 的 SHA256，dll_sha256 为 ZIP 内 driver.dll 的 SHA256。

固定目录下载地址分别为：

- `https://github.com/chenran0408/peripheral-desk-drivers/releases/download/catalog-stable/catalog-stable.json`
- `https://github.com/chenran0408/peripheral-desk-drivers/releases/download/catalog-preview/catalog-preview.json`

签名地址在 JSON 地址后追加 `.sig`。目录中的 ZIP 地址指向不可覆盖的 `v...` 版本发布，而非通道发布。例：本次标签为 v0.4.1-preview.1，即使某个包本身的 version 为 0.4.0，其 package_url 仍指向本次标签实际上传的 ZIP；文件名继续包含包版本。`catalog.json` 是 stable 的兼容别名。

包内 manifest 不包含 ZIP 的 hash、下载 URL 或大小，避免自引用。manifest.device 与 catalog 对应条目的 device 必须逐字段一致；宿主应严格检查 id/version/ABI 和两级 hash。catalog 本身须来自宿主信任的维护者发布地址；hash 只验证内容一致性，不能把任意未知 DLL 变成可信代码。

`matches` 条目间是 OR，同一条目字段间是 AND。可选字段：usage_page、usage、feature_len、product_equals、path_contains。整数按 JSON 数字写，可通过 Python 的 0x... 生成。硬件匹配的最终判定规则以宿主实现为准，勿用 product 名作为唯一标识。

新设备完整可选能力格式见 [模板 metadata.json](../templates/device/metadata.json)。四款已知设备在宿主中保留已有详细能力，drivers.json 此版先记录准确匹配和基本资料。

每个包还必须声明 core_min="0.4.0"、core_max_exclusive="0.5.0"、channel、capabilities 字符串数组和用户向 changelog；catalog 与包内清单逐字段一致。ABI兼容不代表任意主程序版本均支持，宿主必须同时检查版本范围。

`version` 是该设备包自己的版本，不是主程序最低版本。例如稳定包0.4.1与G305预览包0.4.1-preview.1都仍使用ABI 1和上述0.4.x核心范围。只改一款设备时只提高该包的版本并选择该包构建，无需重发其他设备。

版本发布的 catalog 是本批ZIP的子集；固定通道目录才是累计完整目录。`publish-channels.ps1` 会验证旧目录签名、按id合并并重签，保留其余设备的原始元数据和下载地址，拒绝同版不同哈希或倒退。空stable/preview输入不会清空已有通道。不要手工把subset直接覆盖固定通道目录。

## 目录签名与离线安装

主程序只安装维护者审核并签名的 release 目录所列包。catalog JSON 及同名 `.json.sig` 必须来自同一次受信发布；主程序先使用内置 RSA 公钥验证 SHA256/PKCS1 签名，再核对 ZIP 和 DLL hash。离线安装也必须携带该签名目录与 `.sig`，任意 JSON 或 PR 的未签名构建产物均不能作为可信安装来源。包 ZIP 自身不循环包含目录签名。

公钥为 `assets/catalog-public-key.blob`（BCRYPT_RSAPUBLIC_BLOB），可公开。私钥只放本地私密目录或 GitHub Secret `DRIVER_CATALOG_SIGNING_KEY`，绝不进入源码、包或日志。本地维护者用 `scripts/sign-catalog.ps1 -PrivateKeyPath <私钥XML路径>`；也可从同名环境变量读取 XML。`scripts/verify-signatures.ps1` 只需要公钥。

PR 与普通主分支 CI 只构建、核对 ABI 和包格式，不签名、不发布。受审的 main 提交上的现有 tag，或从 main 手动选择同一 tag，才会使用 Secret 签名并发布；发布后 CI 复核公开目录签名与所有包。签名不等于安全审计，维护者仍须检查设备匹配、协议和失败恢复。
