# Logitech G305 只读预览驱动

状态：**公开实现适配未实测**。本机没有 G305，尚未验证 Windows 实机电量、固件差异、休眠唤醒、拔插和与 G HUB 并发。模拟事务、DLL 构建与 ABI 检查不能代替这些结果。

## 精确范围

- 接收器 `046D:C53F`，HID++ 长报文集合 `FF00:0002`，Input/Output 均 20 字节；只寻址配对槽位 1。
- 必须同时读到 HID++ `4.2`、公开 `0x0003` 的无线 transport ID `4074`，以及 `0x0005` 全名 **G305 Lightspeed Wireless Gaming Mouse**。G304 虽也有 `4074`，名称不同，仍拒绝冒认。
- 只查询 ROOT 的协议/特性、公开身份、名称、`0x1000` 电量和公开固件信息。隐藏或已弃用特性拒绝访问，不执行配对、配置、DPI、按键、宏、灯光和固件写入。向 HID 发送查询报文属于只读协议事务，不等于零 USB 输出。
- `0x1000` 的 0 表示百分比不可用，显示未知；大于 100、截断、外来客户端回复、通知、未知电池状态和错误均不生成电量。接收器存在但未确认型号时 `present=false`；已确认型号但电量失败时 `present=true/fresh=false` 并显示原因。
- 原厂电量可能是档位值，例如 90/50/30，并不承诺每 1% 连续变化。没有电压到百分比猜测算法。
- 不支持其他接收器、Bluetooth、共享多槽接收器、G304 和名字相近的其他产品；不能只新增 VID/PID 绕过运行时身份校验。

## 固定依据与固件记录

主要依据为 [Solaar e7304c4](https://github.com/pwr-Solaar/Solaar/tree/e7304c4c451cc9bb4f206a914844525e67856a28) 的 `base.py`（帧与回复）、`hidpp20.py`（公开名称/身份/电量字段）和 `docs/devices/G305 Lightspeed Wireless Gaming Mouse 4074.text`。其公开 G305 记录为 `BOT 69.02.B0021` / `RQM 68.02.B0021`。更早的 [G305 issue 850](https://github.com/pwr-Solaar/Solaar/issues/850) 同时提供 `046D:C53F`、WPID 4074、HID++4.2、电量与 `RQM 68.00.B0011`、`68.01.B0014` 的证据。**这些是上游公开记录，不是本项目实测，也不证明其他固件可用。** 当前 manifest 的 firmware 保持 `unknown`；实际查询成功时状态身份行显示读到的固件。

基础帧与 ROOT ping 另核对 [Logitech 官方 cpg-docs f9107a4](https://github.com/Logitech/cpg-docs/tree/f9107a4e3190d798f9be8c5d596f255f5e5741d2/hidpp20)。Windows HIDClass 分开短/长报文集合的处理，核对 [OpenRGB 763e325 的 Windows 长报文分支](https://github.com/CalcProgrammer1/OpenRGB/blob/763e325f9b8afda33ed5dd9be73a0eee7be42bff/Controllers/LogitechController/LogitechHIDPP20Controller/LogitechHIDPP20Controller.cpp)。未拷贝其控制器实现。

Solaar 对应源码头为 GPL-2.0-or-later；保留 [许可证与出处](../THIRD_PARTY_LICENSES/Solaar/NOTICE.txt)。本项目自行实现 Rust 协议和 Win32 传输，未内嵌 Solaar、Logitech 软件或固件。设备示意图由本项目绘制，不是 G305 官方产品照。

## 结构与打包

`crates/protocol-hidpp` 提供可复用的严格帧匹配、只读 G305 会话和带取消收尾的 Win32 interrupt 传输。`drivers/logitech-g305` 使用 `export_driver!(key, custom_handler)`，只接受 ABI 1 的 `read-state`；不调用核心四设备的 dispatcher。新型号必须建立自己的守卫与说明，不能因为共用 HID++ 就自动加入。

预览 metadata 位于 `drivers-preview.json`，与默认 `drivers.json` 四款稳定驱动隔离。维护者使用固定 SDK revision 后单独构建：

```powershell
python scripts/build.py --release --metadata drivers-preview.json --package logitech-g305 --release-tag v0.4.1-preview.1 --out dist-preview
```

该命令只构建和打包所选驱动；默认命令仍只打包四款原设备。生成 preview 目录仍须走维护者审核、签名及对应预览发布流程；不要把 `dist-preview/catalog-stable.json` 的空目录拿去覆盖现有稳定目录。源码存在不表示公开目录已经上架。

## 本轮验证

已通过 4 项针对性测试：精确名称/WPID 守卫；回复客户端、通知及错误隔离；电量未知/越界；完整模拟只读事务（错误型号不查询电量，确认身份后的电量失败有独立状态）。已构建 Windows x64 release DLL；软件 ABI 检查确认 ABI=1，设置和按键写入请求在枚举 HID 前被拒绝，ZIP 哈希、DLL 哈希、metadata 一致且包含许可证。尚无实机测试。
