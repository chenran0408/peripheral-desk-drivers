# Vigil Watt 设备迁移清单

参考基线：Vigil Watt v1.3.0（commit `a78e973b4eb634bdce37c9078dd48d1c07e3eb73`），核对日期 2026-10-26。清单合并该版本完整协议索引、Release 新增型号和 README 历史支持矩阵中的设备；本批新增 41 个插件已在 [v0.4.13-preview.11](https://github.com/chenran0408/peripheral-desk-drivers/releases/tag/v0.4.13-preview.11) 发布，同一插件按独立型号条目记录，不把相同品牌、VID/PID 或相近代际当成兼容证据。电量插件只移植 Vigil Watt 已实现的读取/识别/充电状态能力；控制项要有对应品牌开源实现及独立来源才开放。

状态：`已发布` 表示已有可下载插件；`未接入` 表示目前没有精确型号插件。上游验证与 Peripheral Desk 本项目实机验证分别标记。上游“贡献者声明”按原文保留，不等同于维护者实测，也不代表本项目实机验证。

| 上游设备 | VID:PID / 路径范围 | Vigil Watt 上游状态 | Peripheral Desk 插件 / 当前状态 | 功能缺项说明 |
|---|---|---|---|---|
| MCHOSE G87 V1 键盘 | `41E4:2001` | 本机验证 | [`mchose-g87-v1`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/mchose-g87-v1-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 电量与充电；改键、宏和灯光没有该通道的已验证写入来源 |
| MCHOSE QHW 新一代 dongle | `3837:3033` | 已实现备用 | 未接入 | 只对官方支持的精确型号/功能映射接入，不推广到旧接收器 |
| MCHOSE G87 V2 键盘 | `3837:201B` | 贡献者真机；键盘走 V1 会话 | [`mchose-g87-v2`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/mchose-g87-v2-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 单独型号条目；仅电量/充电，不能把 G87 V1 V1.0 的其他协议推演过来 |
| MCHOSE A7 V2 Pro 鼠标 | `3837:100A` | 贡献者真机 | [`mchose-a7-v2-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/mchose-a7-v2-pro-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 使用鼠标 V2 被动状态帧，与 A7 V3 Ultra+ 有线接收器协议不同 |
| MCHOSE K7 V2 Ultra+ 鼠标 | `3837:1018` | 贡献者真机 | 未接入 | DPI/回报率/按键/灯效需另有对应控制来源 |
| MCHOSE G3 V2 鼠标 | `3837:4245` | 真机验证 | 未接入 | 独立鼠标代际，不能套用 K7/A5 V2 协议 |
| MCHOSE V9 Pro 耳机 | `291D:385D` | 贡献者真机 | `mchose-v9-pro` / 已有预览插件 | 仅电量；充电状态及音频控制未知 |
| MCHOSE X9 耳机 | `3837:6045` | 贡献者声明 | 未接入 | 电量能力待接入；音效/灯效不由 Vigil Watt 覆盖 |
| MCHOSE A5 V2 Ultra 鼠标 | `3837:100C` | 贡献者声明 | 未接入（与初代 A5 插件分开） | 代际需精确识别；不复用初代 A5 插件身份 |
| MCHOSE G75 Pro 键盘 | `3554:FA09` | 贡献者真机 36% | `mchose-g75-pro` / 已有稳定插件 | 其他同 VID/PID 型号须通过 ACK 识别 |
| LINGBAO M5PRO Max 鼠标 | `3151:402D` | 贡献者真机 88%/95% | [`lingbao-m5pro-max`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/lingbao-m5pro-max-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 仅 2.4 GHz 接收器电量/充电；排除有线回显通道；控制未接入 |
| Logitech Bolt + MX Master 3S | `046D:C548` | 本机验证 | [`logitech-bolt-mx-master-3s`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/logitech-bolt-mx-master-3s-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | Bolt 接收器长 HID++ 通道；先验证已配对设备名精确为 MX Master 3S |
| Logitech PRO X 2 LIGHTSPEED | `046D:0AF7` | 2.4G/蓝牙真机 | [`logitech-pro-x2-lightspeed`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/logitech-pro-x2-lightspeed-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 只支持 2.4 GHz 接收器电量；不含 G HUB EQ、环绕声或麦克风设置 |
| Rapoo VT3 II | `24AE:1412` | 本机验证 | [`rapoo-vt3-ii`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/rapoo-vt3-ii-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动接收器心跳；休眠时无新读数，按未知处理 |
| BETOP T9 | `20BC:5073` | 本机验证 | [`betop-t9`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/betop-t9-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动电量；离线占位值拒绝显示 |
| BETOP 鲲鹏20 | `5159` / `5158` / `5191` | 2.4G/有线贡献者真机；`5191` 未验 | [`betop-kunpeng20`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/betop-kunpeng20-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 2.4G/有线精确设备名；`5191` 变体保持不支持 |
| INPHIC ZHEN 接收器 | `1D57:FA65` | 本机验证 | [`inphic-zhen`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/inphic-zhen-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动报告；与 DELUX 通过不同用法/报告通道区分 |
| HECATE G5M Pro | `35BB:D00B` | 贡献者真机 | [`hecate-g5m-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/hecate-g5m-pro-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 独立主动只读电量/充电查询；与 G2 Pro 分离，无 EQ/灯效 |
| HECATE G3M Pro 升级版（D3FB 接收器） | `35BB:D3FB` | 真机验证 | [`hecate-g3m-pro-upgrade`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/hecate-g3m-pro-upgrade-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 接收器协议无法识别配对鼠标型号，只显示电量；充电字段未知 |
| DELUX M900 Pro | `1D57:FA65` | 贡献者真机 | [`delux-m900-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/delux-m900-pro-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动 000A:0000 五字节接口，和 INPHIC 分流；坞站充电时读数可能失真 |
| AULA F99 Pro | `3554:FA09` | 贡献者真机 | 未接入 | 与 MCHOSE G75、LINGBAO K98 共号段；需会话 ACK 分流 |
| AULA HERO 68 Air | `372E:106C` + UUID | 贡献者真机 10% 放电/充电 | [`aula-hero-68-air`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/aula-hero-68-air-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | UUID 精确识别后的电量/充电；本项目未实机验证 |
| AKG N9 Hybrid | `0ECB:20D9` | 贡献者真机 50% | [`akg-n9-hybrid`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/akg-n9-hybrid-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 只读电量；充电状态、EQ 与其他控制未适配 |
| LINGBAO K98 | `3554:FA09` | 贡献者真机 100% | 未接入 | 与 G75/AULA F99 同 VID/PID，必须 ACK 分流 |
| VGN F1 V5 Turbo+ | `391D:1A23` / `1026` | 贡献者真机 | [`vgn-f1-v5-turbo-plus`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/vgn-f1-v5-turbo-plus-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 有线/接收器电量与充电；未据此开放 DPI、回报率、灯效或改键 |
| VGN F2 Master+ | `3554:FB56` / `FB57` | 贡献者真机 80% | `vgn-f2-master-plus` / 已有预览插件 | DPI、回报率、受限按键映射基于 OpenMouse；其他功能不宣称 |
| AULA F108 Pro | `05AC:024F` | 贡献者真机 | [`aula-f108-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/aula-f108-pro-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 与 GANSS GS3104T-Pro 共用协议及插件；只在产品名明确时分别显示品牌 |
| AULA G9 耳机 | `38A6:0030` | 贡献者声明 | [`aula-g9`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/aula-g9-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动状态；旧固件 0/0 显示在线但电量未知 |
| AULA L9 Ultra 鼠标 | `38A6:28FD` | 贡献者声明 | [`aula-l9-ultra`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/aula-l9-ultra-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 主动读取电量/充电；无 DPI、回报率、改键、宏或灯光 |
| VGN V98 Pro 键盘 | `320F:5088` | 贡献者声明 | [`vgn-v98-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/vgn-v98-pro-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 只读电量和已知充电标志；无键位、宏和灯光控制 |
| Sony DualSense 蓝牙 | `054C:0CE6` | 贡献者真机 | [`sony-dualsense-bluetooth`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/sony-dualsense-bluetooth-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动监听 CRC32 报告；电量只有 10% 档位，按上游整档估算显示；不适用于 USB/Edge |
| Haute42 Adapter 4DS | `054C:0CE6`（借用 DualSense ID） | 贡献者声明 | [`haute42-adapter-4ds`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/haute42-adapter-4ds-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 必须确认 Haute42 固件字符串；普通 DualSense 不匹配；只读 |
| GANSS GS3104T-Pro | `05AC:024F` | 贡献者真机 | `aula-f108-pro` / 共享插件源码已集成，待发布 | 上游记录两款指纹相同；同一插件避免并发查询 |
| CHERRY PIXIU75 | `046A:01E4` / `01E2` | 贡献者真机 | [`cherry-pixiu75`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/cherry-pixiu75-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 覆盖无线接收器与 USB 有线；只读电量/充电 |
| PULSAR X2 V2 | `3554:F508` | 贡献者真机 | [`pulsar-x2-v2`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/pulsar-x2-v2-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 仅 1 kHz 接收器；休眠时显示未知电量，不套用 4K/有线型号 |
| DARKRIM SHADOW | `3654:6901` | 贡献者真机 | [`razer-darkrim-shadow`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/razer-darkrim-shadow-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 校验 CRC/TLV；官方程序运行或状态不明时不打开 HID |
| GameSir G7 Pro CE | `3537:10B9` / `0575` | 贡献者真机 | [`gamesir-g7-pro-ce`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/gamesir-g7-pro-ce-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | FFF0:0040 64 字节主动只读心跳；仅匹配上游记录的两个修订号 |
| ATK/VXE F1 Pro Max | `373B:1015` / `101A` / `101B` | 本机验证 | [`atk-f1-pro-max`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/atk-f1-pro-max-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 电量、DPI、回报率；8K 仅 101B 接收器；无改键/宏/灯光 |
| ATK F1 V3 ULTIMATE+ | `373B:1303` 8K-M 接收器；`373B:12A9` 有线 | 接收器实机报告；有线报告字段存在、长度未实测 | [`atk-f1-v3-ultimate-plus`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/atk-f1-v3-ultimate-plus-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 电量、充电、电压；只读 |
| ATK F1 Ultimate 2.0 | `373B:11D9` / `11E4` | 上游映射与通用协议代码；无型号专属实机报告 | [`atk-f1-ultimate-2-0`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/atk-f1-ultimate-2-0-0.4.11-preview.1-windows-x64.zip) / 已发布预览，型号协议未验证 | 只读电量；预览状态明确标注型号协议未验证 |
| ATK A98 Pro 键盘（VIA） | `373B:1152` | 贡献者声明 | [`atk-a98-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/atk-a98-pro-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 仅电量；缺少精确布局、keymap、灯效和宏读写来源，控制保持关闭 |
| HyperX Cloud Flight 2 | `03F0:0AC1` | 本机验证 | [`hyperx-cloud-flight-2`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/hyperx-cloud-flight-2-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 被动电量/充电/连接状态；不含音效或灯效 |
| Razer Viper V4 Pro | `1532:00E6` / `00E5` | 贡献者真机 100% | [`razer-viper-v4-pro`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/razer-viper-v4-pro-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 电量、DPI、回报率、休眠；没有经确认的改键、宏或灯效适配 |
| Razer BlackShark V2 Pro (2023) | `1532:0555` | 贡献者真机 77% | [`razer-blackshark-v2-pro-2023`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/razer-blackshark-v2-pro-2023-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 需启用主动读取；仅电量/充电，无 EQ、音量或麦克风控制 |
| Razer Viper V3 Pro SE | `1532:00DF` | 贡献者真机 58% | [`razer-viper-v3-pro-se`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/razer-viper-v3-pro-se-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 接收器电量、DPI、125/500/1000 Hz；无休眠、改键、宏或灯效 |
| Razer Kraken V4 | `1532:056C` | 贡献者声明 | [`razer-kraken-v4`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/razer-kraken-v4-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 只读电量/充电；灯光与音频控制未适配 |
| ASUS ROG Strix Scope II 96 Omni | `0B05:1ACE` | 贡献者真机 95% | [`asus-rog-scope-ii-96-omni`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/asus-rog-scope-ii-96-omni-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | Omni 接收器配对端只读电量；未声明 Omni RGB 控制 |
| EWEADN X87 键盘 | `0C45:FEFE` | 贡献者真机 | [`eweadn-x87`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/eweadn-x87-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 限定 MI_03 33 字节接口；排除 MI_04 |
| EWEADN S9 Pro/Max/Ultra 鼠标 | `372E:1026` | 贡献者真机 | [`eweadn-s9-series`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/eweadn-s9-series-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 共享协议明示覆盖 Pro/Max/Ultra；只读电量/充电 |
| AJAZZ AK870 | `3151:4011` | 贡献者真机 | [`ajazz-ak870`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/ajazz-ak870-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 键盘身份确认后只读电量；不等于 AJ179 PRO 插件 |
| THUNDEROBOT ML903 NearLink Se | `369B:0234` | 贡献者真机 | [`thunderobot-ml903-nearlink-se`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/thunderobot-ml903-nearlink-se-0.4.11-preview.1-windows-x64.zip) / 已发布 v0.4.13-preview.11 | 只读接收器电量 TLV；不把工作模式误作电量或充电 |
| ROSE FIRST ANGEL | `38FF:0001` | 2.4G 无电量通道 | 不做电量插件 | 上游已确认该连接没有电量读取能力 |
| EDIFIER W820NB（经典蓝牙） | Windows HFP 网关节点 `DEVPKEY_Device_BatteryLevel` | Vigil Watt 本机验证 | 暂未接入插件 | 上游走 Windows PnP/HFP 设备节点，不是 HID；当前 Peripheral Desk 插件发现接口没有经典蓝牙设备节点枚举，需主程序提供系统电池来源后才能按设备展示 |
| Sony INZONE H9 II | `054C:0FA8` | 维护者实物验证 | [`sony-inzone-h9-ii`](https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v0.4.13-preview.11/sony-inzone-h9-ii-0.1.0-windows-x64.zip) / 已发布 v0.4.13-preview.11 | USB 接收器电量与充电状态；不含 EQ、灯光或音量设置 |

## 许可与来源

- Vigil Watt 协议事实基于 [v1.3.0 硬件索引](https://github.com/evren-oct/vigil-watt/blob/a78e973/docs/hardware/protocols.md) 与对应型号页面；每个插件需记录精确型号、连接方式及上游验证状态。
- Vigil Watt 源码采用 MIT，复用代码时保留原版权与许可证文本；仅参考协议事实不复制代码时不把协议文档描述成代码来源。
- 控制功能来源另列在各插件 `README.md` / `THIRD_PARTY_LICENSES`。Vigil Watt 是电量工具，不单独证明 DPI、改键、宏、灯效或音效功能。
- 所有插件按需安装，主程序不捆绑设备实现；未完成项不在 UI 里显示成可用能力。
