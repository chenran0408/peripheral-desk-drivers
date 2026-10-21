# G2 Pro 灯光接口依据

仅匹配 USB `35BB:A100`、usage page `0x000C`、FeatureReportByteLength `16` 的唯一接口。本次开关读→切换→恢复已实机通过；没有写模式、RGB 或速度，也不宣称这些项目已经通过实机验证。设备当前只确认了开关读回，界面模式与颜色必须标明为本工具保存的配置。

## 官方程序

资料来自用户提供的 HECATE 1.1.44 安装包内 HECATE.exe，SHA256 为 `a1bd42c16dd46c6a86975b70ce79e658c035d2488ec09913185182d4a6086e3f`。只记录协议语义，不附带或重新分发厂商程序。

- `LedEndpoint_G2Pro` vtable 的设置入口 `0x140375540`：先调用 `hid_write` 发送逻辑字节 `ED 0B`，再用 `hid_send_feature_report` 发送 `ED 02 0B <0或1>`。
- 回复处理入口 `0x140376740`：同样选择 `ED 0B`，调用 `hid_get_feature_report`，严格要求至少四字节，头为 `ED 02 0B`，第四字节只能是 `0` 或 `1`。

## Windows 传输差异

官方 HIDAPI 的 `hid_send_feature_report` 接受逻辑四字节后，会补零到 HID 描述符要求的 FeatureReportByteLength，再调用 Win32 `HidD_SetFeature`。直接调用 Win32 时也必须发送完整 16 字节。此前直接发送长度 4 漏掉了这一层，现已修正。

参考固定提交 [HIDAPI windows/hid.c](https://github.com/libusb/hidapi/blob/852cc68b8e0e4c7eba0815ef992951cd8b75c55b/windows/hid.c#L1171)，以及该提交的 [许可说明](https://github.com/libusb/hidapi/blob/852cc68b8e0e4c7eba0815ef992951cd8b75c55b/LICENSE.txt)：GPLv3、BSD-style 或原 HIDAPI 许可可选。这里只借鉴 Windows API 的补齐语义，由项目自行实现 Rust 传输，没有复制或随包分发 HIDAPI 源码或 DLL。

读取遇到未识别报文时仅做三次有界只读重试；始终保留官方头与布尔值校验，不能把未知回复解释成“关闭”。写入不重发。实机记录确认原状态关闭，切换开启后读到开启，恢复后读到关闭，模式与 RGB 未写入。

## 0.4.2 即时开关与界面恢复

插件新增 `lighting-switch` 能力和请求：`{"op":"lighting-switch","enabled":true}`。只操作 `ED 0B` / `ED 02 0B <bool>`，不发送 `FF` 模式、色表或速度。写入前读原开关，写后核对，失败尝试恢复原值并报告结果；不能读取基线时拒绝盲写。

本轮补充测试从耳罩亮起开始，发送关闭后用户确认两侧实际熄灭，随后恢复开启并读回。没有再次写入颜色或模式。硬件关闭是有效状态，不能因此禁用“应用并开灯”；读取失败须保留重新读取和尝试开灯入口。本地配置损坏仅提示并保留原文件，不应让可读的硬件开关变成未知。合法配置的未知字段保留；首次开关不创建臆测的默认灯效。

历史灰按钮可由初次读取失败后的界面状态复现，但没有用户当次错误日志，不能断言必由本地配置损坏引起。旧插件没有该能力时，主程序提示更新，仍保留原有应用入口。
