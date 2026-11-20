# 罗技 G 系列板载配置

在「插件管理」安装对应型号的预览插件并打开「设备调节 → 板载配置」。可以查看设备当前由主机还是板载配置控制，切换模式，选择已启用的配置，以及切换其中已有的 DPI 档位。配置和档位均从鼠标读取。G305 在主机模式还可以使用原有的 DPI 调节。图形编辑器需要外设盒 0.4.11-preview.17 或更新版本。

| 鼠标 | LIGHTSPEED 接收器 VID:PID | 需要匹配的 WPID 与设备全名 | 上游设备记录 |
| --- | --- | --- | --- |
| G304 | 046D:C53F | 4074 · G304 Lightspeed Wireless Gaming Mouse | [Solaar G304](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/docs/devices/G304%20Lightspeed%20Wireless%20Gaming%20Mouse%204074.txt) |
| G305 | 046D:C53F | 4074 · G305 Lightspeed Wireless Gaming Mouse | [Solaar G305](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/docs/devices/G305%20Lightspeed%20Wireless%20Gaming%20Mouse%204074.text) |
| G502 LIGHTSPEED | 046D:C539 / C547 | 407F · G502 LIGHTSPEED Wireless Gaming Mouse | [Solaar G502 LIGHTSPEED](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/docs/devices/G502%20Lightspeed%20Wireless%20Gaming%20Mouse%20407F.txt) |
| G502 X PLUS | 046D:C539 / C547 | 4099 · G502 X PLUS | [Solaar G502 X PLUS](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/docs/devices/G502%20X%20PLUS%204099.txt) |

四份上游实物记录均列出 HID++ 0x8100 v0 及设备的板载模式读数。接收器范围沿用已有型号插件，板载接口在接收器槽 1 核对配对设备自报的 WPID 和全名。切换配置后核对读回值，发生失败则尝试恢复此前的模式、配置和已知的活动 DPI 档位。G304/G305 即使共用接收器和 WPID，也按完整型号名称分别安装。当前插件适用上表中的 2.4 GHz 连接。USB 数据线、其他接收器、G304 X/G305 X 和其他 G502 变种需要各自适配。

本批只切换已保存的配置和 DPI 档位。改写板载槽内的 DPI 数值、创建配置、改键、宏、灯光及回报率不在本批范围内。板载配置是按需查询，后台电量刷新继续走原接口。三项协议样本测试覆盖主机/板载选择、出厂区、空 DPI 档、切换后读回及确认包丢失时恢复原配置和 DPI。本项目这次没有在这四款鼠标上执行硬件写入。

## 实现和来源

罗技 OMM 提供操作流程参考。外设盒使用自主编写的 Rust HID++ 插件，与已有原生入口配合；安装包中没有 OMM 的可执行文件或专有资源。[罗技 OMM 说明](https://support.logi.com/hc/en-us/articles/38103470390935-Onboard-Memory-Manager) · [板载模式](https://support.logi.com/hc/en-001/articles/360057433714-How-to-enable-On-Board-Memory-Mode-to-use-Onboard-Memory-Manager) · [G502 LIGHTSPEED 官方手册](https://www.logitech.com/assets/65648/3/g502-lightspeed.pdf)

Solaar 修订 `e7304c4c451cc9bb4f206a914844525e67856a28` 提供型号报告、运行模式、配置槽 ID 和板载布局的协议依据；libratbag 修订 `8235b5bb6032dea15901ab58a8218956f4494d08` 提供活动 DPI 的读取与切换、格式及 G305 空用户扇区处理的协议依据。这里参考协议事实，没有移植两库源码。[Solaar 板载选择](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/lib/logitech_receiver/settings_templates.py) · [Solaar 布局](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/lib/logitech_receiver/hidpp20.py) · [Solaar 许可](https://github.com/pwr-Solaar/Solaar/blob/e7304c4c451cc9bb4f206a914844525e67856a28/LICENSE.txt) · [libratbag HID++](https://github.com/libratbag/libratbag/blob/8235b5bb6032dea15901ab58a8218956f4494d08/src/hidpp20.c) · [libratbag 许可](https://github.com/libratbag/libratbag/blob/8235b5bb6032dea15901ab58a8218956f4494d08/COPYING)
