# Vigil Watt 厂商研究对照

稳定版 [v1.3.1](https://github.com/evren-oct/vigil-watt/releases/tag/v1.3.1) 与开发版 [6315988](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471) 的[完整差异](https://github.com/evren-oct/vigil-watt/compare/4df830043296760f5d5c273f4cdc0ea9456c4bc7...6315988f54e47fc030ae4c535dc64ae27c08f471)。

## 上游厂商实现

| 上游模块 | 有变化的文件 | 稳定版至开发版的结构化变动 |
| --- | ---: | --- |
| [Ajazz](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Ajazz) | 1 | — |
| [Akg](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Akg) | 2 | — |
| [Asus](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Asus) | 1 | — |
| [Atk](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Atk) | 4 | — |
| [Aula](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Aula) | 6 | — |
| [Betop](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Betop) | 7 | +179 |
| [Cherry](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Cherry) | 2 | — |
| [Compx](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Compx) | 1 | — |
| [Darkrim](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Darkrim) | 2 | — |
| [Delux](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Delux) | 3 | — |
| [Eweadn](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Eweadn) | 3 | — |
| [Flydigi](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Flydigi) | 2 | — |
| [GameSir](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/GameSir) | 2 | — |
| [Ganss](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Ganss) | 2 | — |
| [Haute](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Haute) | 2 | — |
| [Hecate](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Hecate) | 3 | — |
| [HyperX](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/HyperX) | 2 | — |
| [Ilovbee](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Ilovbee) | 2 | — |
| [Inphic](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Inphic) | 4 | — |
| [Langtu](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Langtu) | 2 | — |
| [Lingbao](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Lingbao) | 2 | — |
| [Logitech](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Logitech) | 11 | ~23 |
| [Mchose](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose) | 17 | +105 |
| [Mojhon](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mojhon) | 2 | — |
| [Pulsar](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Pulsar) | 2 | — |
| [Rapoo](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Rapoo) | 6 | +207 |
| [Razer](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Razer) | 3 | — |
| [Sony](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Sony) | 4 | — |
| [Thunderobot](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Thunderobot) | 1 | — |
| [Vgn](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Vgn) | 4 | — |

变动符号依次代表新登记、内容修改和移除。以下链接列出厂商的协议、采样和设备范围。

[asus-rog](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/asus-rog.md) · [atk](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/atk.md) · [aula](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/aula.md) · [betop](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/betop.md) · [cherry](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/cherry.md) · [darkrim](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/darkrim.md) · [delux](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/delux.md) · [eweadn](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/eweadn.md) · [flydigi](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/flydigi.md) · [gamesir](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/gamesir.md) · [ganss](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/ganss.md) · [hecate](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/hecate.md) · [hyperx-cf2](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/hyperx-cf2.md) · [inphic](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/inphic.md) · [lingbao](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/lingbao.md) · [logitech](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/logitech.md) · [mchose](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/mchose.md) · [mojhon](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/mojhon.md) · [pulsar](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/pulsar.md) · [rapoo](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/rapoo.md) · [razer](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/razer.md) · [rose](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/rose.md) · [vgn](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/vgn.md)

## 型号与接口

每行是一条上游身份。插件链接只覆盖链接中注明的接口和连接方式。共用接收器一栏列出本库声明同一硬件标识的插件数，具体型号由设备认领和插件判断。“待对照”可用于挑选下一项适配，完整型号列表也可在[本库厂商目录](../../README.md)查找。

### betop

[上游固定版本](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Betop/betop-catalog.json) · [目录原文](betop-catalog.json)

| 上游型号或通道 | VID:PID | 协议 / 连接方式 | 本库对应 |
| --- | --- | --- | --- |
| 宙斯2 | 20BC:5073 | betop-zeus-keyevent · controller | [betop-t9](../../../drivers/betop-t9) · 宙斯 2 被动广播 |
| 未命名 | 20BC:5125 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏20智控游戏手柄 | 20BC:5126 | betop-status15-nocharge · controller | 待对照 |
| 未命名 | 20BC:5127 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏20智控游戏手柄 | 20BC:5128 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏20 绝尘白 | 20BC:5158 | betop-status15 · controller | [betop-kunpeng20](../../../drivers/betop-kunpeng20) · 鲲鹏 20 有线 |
| 鲲鹏20 绝尘白 | 20BC:5159 | betop-status15 · controller | [betop-kunpeng20](../../../drivers/betop-kunpeng20) · 鲲鹏 20 接收器 |
| 鲲鹏20战双帕弥什联名款 | 20BC:516F | betop-status15 · controller | 待对照 |
| 鲲鹏20战双帕弥什联名款 | 20BC:5170 | betop-status15 · controller | 待对照 |
| 鲲鹏20精英版-丁未 | 20BC:5190 | betop-status15 · controller | 待对照 |
| 鲲鹏20精英版-丁未 | 20BC:5191 | betop-status15 · controller | 待对照 |
| 未命名 | 20BC:5145 | betop-status15-nocharge · controller | 待对照 |
| 未命名 | 20BC:5146 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 明末联动款 | 20BC:5154 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 明末联动款 | 20BC:5155 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 御铠白 | 20BC:515B | betop-status15 · controller | 待对照 |
| 鲲鹏40 御铠白 | 20BC:515C | betop-status15 · controller | 待对照 |
| 鲲鹏40智控游戏手柄 风驰白 | 20BC:515D | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40智控游戏手柄 风驰白 | 20BC:515E | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40鸣潮联名款 | 20BC:5163 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40鸣潮联名款 | 20BC:5164 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40鸣潮联名款 | 20BC:5165 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40鸣潮联名款 | 20BC:5166 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 电掣黑 | 20BC:5169 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 电掣黑 | 20BC:516A | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 精英版 | 20BC:5194 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 精英版 | 20BC:5195 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50 云翼白 | 20BC:5133 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50 星穹紫 | 20BC:5149 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20BC:5150 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20BC:5151 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20BC:5152 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20BC:5153 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50 云翼白 | 20BC:5134 | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏50 星穹紫 | 20BC:514A | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏70 AI旗舰 | 20BC:512F | betop-status15 · controller | 待对照 |
| 鲲鹏70 AI旗舰 | 20BC:5130 | betop-status15 · controller | 待对照 |
| 鲲鹏70AI智控旗舰手柄 | 20BC:515F | betop-status15 · controller | 待对照 |
| 鲲鹏70AI智控旗舰手柄 | 20BC:5160 | betop-status15 · controller | 待对照 |
| 鲲鹏70 午马限定款 | 20BC:516D | betop-status15 · controller | 待对照 |
| 鲲鹏70 午马限定款 | 20BC:516E | betop-status15 · controller | 待对照 |
| 鲲鹏70 6D升级款 | 20BC:5173 | betop-status15 · controller | 待对照 |
| 鲲鹏70 6D升级款 | 20BC:5174 | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:518A | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:518B | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:518C | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:518D | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:519B | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:519C | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:519D | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20BC:519E | betop-status15 · controller | 待对照 |
| 鲲鹏20智控游戏手柄 | 20DD:5128 | betop-status15 · dongle | 待对照 |
| 鲲鹏70 AI旗舰 | 20DD:5130 | betop-status15 · dock | 待对照 |
| 鲲鹏40 御铠白 | 20DD:515C | betop-status15 · dongle | 待对照 |
| 鲲鹏40鸣潮联名款 | 20DD:5163 | betop-status15 · controller | 待对照 |
| 鲲鹏40鸣潮联名款 | 20DD:5164 | betop-status15 · dongle | 待对照 |
| 鲲鹏40鸣潮联名款 | 20DD:5165 | betop-status15 · controller | 待对照 |
| 鲲鹏40鸣潮联名款 | 20DD:5166 | betop-status15 · dongle | 待对照 |
| 鲲鹏70 午马限定款 | 20DD:516D | betop-status15 · controller | 待对照 |
| 鲲鹏70 午马限定款 | 20DD:516E | betop-status15 · dongle | 待对照 |
| 鲲鹏20战双帕弥什联名款 | 20DD:516F | betop-status15 · controller | 待对照 |
| 鲲鹏20战双帕弥什联名款 | 20DD:5170 | betop-status15 · dongle | 待对照 |
| 鲲鹏70 6D升级款 | 20DD:5173 | betop-status15 · controller | 待对照 |
| 鲲鹏70 6D升级款 | 20DD:5174 | betop-status15 · dongle | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:518A | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:518B | betop-status15 · dongle | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:518C | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:518D | betop-status15 · dongle | 待对照 |
| 鲲鹏20精英版-丁未 | 20DD:5190 | betop-status15 · controller | 待对照 |
| 鲲鹏40 精英版 | 20DD:5194 | betop-status15 · controller | 待对照 |
| 鲲鹏40 精英版 | 20DD:5195 | betop-status15 · dongle | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:519B | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:519C | betop-status15 · dongle | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:519D | betop-status15 · controller | 待对照 |
| 鲲鹏70 异环联名款 | 20DD:519E | betop-status15 · dongle | 待对照 |
| 宙斯 Pro | 20BC:500B | betop-zeus-keyevent · controller | 待对照 |
| 宙斯 Pro | 20BC:502F | betop-zeus-keyevent · controller | 待对照 |
| 未命名 | 20BC:5074 | betop-zeus-keyevent · controller | 待对照 |
| 宙斯2 | 20BC:5076 | betop-zeus-keyevent · controller | 待对照 |
| 宙斯2 | 20BC:5077 | betop-zeus-keyevent · controller | 待对照 |
| 永劫无间 | 20BC:5104 | betop-zeus-keyevent · controller | 待对照 |
| 永劫无间 | 20BC:5105 | betop-zeus-keyevent · controller | 待对照 |
| 永劫无间 | 20BC:5106 | betop-zeus-keyevent · controller | 待对照 |
| 永劫无间 | 20BC:5107 | betop-zeus-keyevent · controller | 待对照 |
| 未命名 | 20DD:5125 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏20智控游戏手柄 | 20DD:5126 | betop-status15-nocharge · controller | 待对照 |
| 未命名 | 20DD:5127 | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏70 AI旗舰 | 20DD:512F | betop-status15 · controller | 待对照 |
| 鲲鹏50 云翼白 | 20DD:5133 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50 云翼白 | 20DD:5134 | betop-status15-nocharge · dongle | 待对照 |
| 未命名 | 20DD:5145 | betop-status15-nocharge · controller | 待对照 |
| 未命名 | 20DD:5146 | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏50 星穹紫 | 20DD:5149 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50 星穹紫 | 20DD:514A | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20DD:5150 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20DD:5151 | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20DD:5152 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏50智控游戏手柄 解限机联名款 | 20DD:5153 | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏40 明末联动款 | 20DD:5154 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 明末联动款 | 20DD:5155 | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏20 绝尘白 | 20DD:5158 | betop-status15 · controller | 待对照 |
| 鲲鹏20 绝尘白 | 20DD:5159 | betop-status15 · dongle | 待对照 |
| 鲲鹏40 御铠白 | 20DD:515B | betop-status15 · controller | 待对照 |
| 鲲鹏40智控游戏手柄 风驰白 | 20DD:515D | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40智控游戏手柄 风驰白 | 20DD:515E | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏70AI智控旗舰手柄 | 20DD:515F | betop-status15 · controller | 待对照 |
| 鲲鹏70AI智控旗舰手柄 | 20DD:5160 | betop-status15 · dongle | 待对照 |
| 鲲鹏40 电掣黑 | 20DD:5169 | betop-status15-nocharge · controller | 待对照 |
| 鲲鹏40 电掣黑 | 20DD:516A | betop-status15-nocharge · dongle | 待对照 |
| 鲲鹏20精英版-丁未 | 20DD:5191 | betop-status15 · dongle | 待对照 |

另有 66 条限于官方登记范围的 PID，可在目录原文的 officialDevices 中按型号查询。

### logitech

[上游固定版本](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Logitech/logitech-hidpp.json) · [目录原文](logitech-hidpp.json)

| 上游型号或通道 | VID:PID | 协议 / 连接方式 | 本库对应 |
| --- | --- | --- | --- |
| Unifying | 046D:C52B | unifying | 同标识候选 13 款 |
| Unifying | 046D:C532 | unifying | 同标识候选 12 款 |
| 27MHz | 046D:C517 | 27mhz | 待对照 |
| Nano | 046D:C518 | nano | 待对照 |
| Nano | 046D:C51A | nano | 待对照 |
| Nano | 046D:C51B | nano | 待对照 |
| Nano | 046D:C521 | nano | 待对照 |
| Nano | 046D:C525 | nano | 待对照 |
| Nano | 046D:C526 | nano | 待对照 |
| Nano | 046D:C52E | nano | 同标识候选 4 款 |
| Nano | 046D:C52F | nano | 待对照 |
| Nano | 046D:C531 | nano | 同标识候选 4 款 |
| Nano | 046D:C534 | nano | 同标识候选 4 款 |
| Nano | 046D:C537 | nano | 同标识候选 4 款 |
| Nano | 046D:C542 | nano | 待对照 |
| LIGHTSPEED | 046D:C539 | lightspeed | 同标识候选 8 款 |
| LIGHTSPEED | 046D:C53A | lightspeed | 同标识候选 5 款 |
| LIGHTSPEED | 046D:C53D | lightspeed | 同标识候选 5 款 |
| LIGHTSPEED | 046D:C53F | lightspeed | 同标识候选 7 款 |
| LIGHTSPEED | 046D:C541 | lightspeed | 同标识候选 5 款 |
| LIGHTSPEED | 046D:C545 | lightspeed | 同标识候选 5 款 |
| LIGHTSPEED | 046D:C547 | lightspeed | 同标识候选 8 款 |
| Bolt | 046D:C548 | bolt | 同标识候选 3 款 |

### mchose

[上游固定版本](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/mchose-catalog.json) · [目录原文](mchose-catalog.json)

| 上游型号或通道 | VID:PID | 协议 / 连接方式 | 本库对应 |
| --- | --- | --- | --- |
| Ace 60 Pro | 19F5:FF0F | qhw | 待对照 |
| Ace 60 | 19F5:FF1B | qhw | 待对照 |
| V9 Pro | 291D:385D | headset-v9 | [mchose-v9-pro](../../../drivers/mchose-v9-pro) · V9 Pro 专用接口 |
| G75 Pro | 3554:FA09 | keyboard-v1 | [mchose-g75-pro](../../../drivers/mchose-g75-pro) · G75 Pro 会话身份 |
| K87S | 3837:200F | qhw | 待对照 |
| G87 V2 | 3837:2019 | qhw | 待对照 |
| K99 V3 | 3837:201F | qhw | 待对照 |
| G75 V2 | 3837:2022 | qhw | 待对照 |
| K87 V3 | 3837:2023 | qhw | 待对照 |
| K87 V3 | 3837:2024 | qhw | 待对照 |
| G98 V3 | 3837:2025 | qhw | 待对照 |
| G98 V3 | 3837:2026 | qhw | 待对照 |
| Ace 68 GT | 3837:3009 | qhw | 待对照 |
| Ace 68 Air 2 | 3837:300B | qhw | 待对照 |
| God 60 | 3837:301A | qhw | 待对照 |
| Ace 60 Pro Nordic | 3837:3022 | qhw | 待对照 |
| Ace 68 V2 III | 3837:3025 | qhw | 待对照 |
| Ace 68 Turbo 16K | 3837:3027 | qhw | 待对照 |
| Ace 68 Turbo 8K | 3837:3029 | qhw | 待对照 |
| Ace 68 III | 3837:302A | qhw | 待对照 |
| Mix 87 III | 3837:302C | qhw | 待对照 |
| GT 75 | 3837:302F | qhw | 待对照 |
| Ace 75 8K | 3837:303D | qhw | 待对照 |
| Ace 60 Pro ISO FR | 3837:3041 | qhw | 待对照 |
| A5 V3 Ultra+ | 3837:4026 | qhw | 待对照 |
| K7 V2 Pro+ | 3837:4027 | mchose-0x4d | [mchose-k7-v2-pro-plus](../../../drivers/mchose-k7-v2-pro-plus) · 3837:1018 接收器上报的鼠标身份 |
| K7 V2 Ultra+ | 3837:4028 | mchose-0x4d | [mchose-k7-v2-ultra-plus](../../../drivers/mchose-k7-v2-ultra-plus) · 3837:1018 接收器上报的鼠标身份 |
| G3 V3 | 3837:4029 | qhw | 待对照 |
| V7 | 3837:402A | qhw | 待对照 |
| A7 V3 | 3837:4030 | qhw | [mchose-a7-v3](../../../drivers/mchose-a7-v3) · 3837:1018 接收器上报的鼠标身份 |
| A7 V3 Pro | 3837:4031 | qhw | [mchose-a7-v3-pro](../../../drivers/mchose-a7-v3-pro) · 3837:1018 接收器上报的鼠标身份 |
| A7 V3 Pro+ | 3837:4032 | qhw | [mchose-a7-v3-pro-plus](../../../drivers/mchose-a7-v3-pro-plus) · 3837:1018 接收器上报的鼠标身份 |
| A7 V3 Ultra+ | 3837:4033 | qhw | 待对照 |
| A5 V3 Ultra+(3955) | 3837:4034 | qhw | [mchose-a5-v3-ultra-plus-3955](../../../drivers/mchose-a5-v3-ultra-plus-3955) · 3837:1018 接收器上报的鼠标身份 |
| A5 V3 Pro | 3837:4035 | qhw | [mchose-a5-v3-pro](../../../drivers/mchose-a5-v3-pro) · 3837:1018 接收器上报的鼠标身份 |
| R7 Ultra | 3837:4036 | qhw | [mchose-r7-ultra](../../../drivers/mchose-r7-ultra) · 3837:1018 接收器上报的鼠标身份 |
| K5 Pro | 3837:4037 | qhw | [mchose-k5-pro](../../../drivers/mchose-k5-pro) · 3837:1018 接收器上报的鼠标身份 |
| K5 Ultra | 3837:4038 | qhw | [mchose-k5-ultra](../../../drivers/mchose-k5-ultra) · 3837:1018 接收器上报的鼠标身份 |
| A7 V3-X | 3837:403A | qhw | [mchose-a7-v3-x](../../../drivers/mchose-a7-v3-x) · 3837:1018 接收器上报的鼠标身份 |
| K7 V2 Pro | 3837:403B | mchose-0x4d | [mchose-k7-v2-pro](../../../drivers/mchose-k7-v2-pro) · 3837:1018 接收器上报的鼠标身份 |
| K7 V2 Ultra | 3837:403C | mchose-0x4d | [mchose-k7-v2-ultra](../../../drivers/mchose-k7-v2-ultra) · 3837:1018 接收器上报的鼠标身份 |
| A7 V3 Ultra | 3837:403D | qhw | [mchose-a7-v3-ultra](../../../drivers/mchose-a7-v3-ultra) · 3837:1018 接收器上报的鼠标身份 |
| V7 Pro | 3837:403E | qhw | 待对照 |
| A7 V3 Magnetic | 3837:403F | qhw | [mchose-a7-v3-magnetic](../../../drivers/mchose-a7-v3-magnetic) · 3837:1018 接收器上报的鼠标身份 |
| K5 V2 | 3837:4040 | qhw | [mchose-k5-v2](../../../drivers/mchose-k5-v2) · 3837:101C 接收器上报的鼠标身份 |
| G3 S | 3837:4041 | qhw | 待对照 |
| A5 V3 Ultra | 3837:4042 | qhw | [mchose-a5-v3-ultra](../../../drivers/mchose-a5-v3-ultra) · 3837:1018 接收器上报的鼠标身份 |
| G3 V2 | 3837:4245 | gmouse | [mchose-g3-v2](../../../drivers/mchose-g3-v2) · G3 V2 的 2.4 GHz 接收器 |
| X9 | 3837:6045 | headset-x9 | [mchose-x9](../../../drivers/mchose-x9) · X9 耳机的专用接口 |
| G87 | 41E4:2001 | keyboard-v1 | [mchose-g87-v1](../../../drivers/mchose-g87-v1) · G87 V1 接收器 |
| Ace 60 | 41E4:2102 | qhw | 待对照 |
| Ace 60 Pro | 41E4:2104 | qhw | 待对照 |
| Ace 60X II | 41E4:2113 | qhw | 待对照 |
| Ace 68 I | 41E4:2115 | qhw | 待对照 |
| Ace 68 Pro | 41E4:2117 | qhw | 待对照 |
| Jet 75 I | 41E4:2119 | qhw | 待对照 |
| Jet 75 II | 41E4:211B | qhw | 待对照 |
| Zero75X | 41E4:211D | qhw | 待对照 |
| Ace 68 Air II | 41E4:2121 | qhw | 待对照 |
| Mix 87 I | 41E4:2123 | qhw | 待对照 |
| Ace 60X I | 41E4:2127 | qhw | 待对照 |
| Ace 68 Air III | 41E4:2133 | qhw | 待对照 |
| A5 V2 Ultra | 3837:100C | mouse-a5 | 同标识候选 1 款 |
| 接收器 | 3837:1014 | mchose-0x4d · dongle-wireless | 待对照 |
| 接收器 | 3837:1016 | mchose-0x4d · dongle-wireless | 待对照 |
| 接收器 | 3837:1018 | mchose-0x4d · dongle-wireless | 同标识候选 17 款 |
| 接收器 | 3837:101C | mchose-0x4d · dongle-wireless | 同标识候选 1 款 |
| 接收器 | 3837:101D | mchose-0x4d · dongle-wireless | 待对照 |
| 接收器 | 3837:100C | mouse-a5 · dongle-wireless | 同标识候选 1 款 |
| 接收器 | 3837:3033 | cz-block · dongle-cz | [mchose-g87-v2-cz](../../../drivers/mchose-g87-v2-cz) · G87 V2 的 CZ 接收器，手动认领 |
| 接收器 | 41E4:2001 | keyboard-v1 · dongle-wireless | [mchose-g87-v1](../../../drivers/mchose-g87-v1) · G87 V1 接收器 |

### rapoo

[上游固定版本](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Rapoo/rapoo-catalog.json) · [目录原文](rapoo-catalog.json)

| 上游型号或通道 | VID:PID | 协议 / 连接方式 | 本库对应 |
| --- | --- | --- | --- |
| A83 Air | 24AE:1914 | telink-keyboard · 无线 | 待对照 |
| A83 Air | 24AE:4814 | telink-keyboard · 有线 | 待对照 |
| Ace 68 Turbo | 3837:3026 | gcome-ap | 待对照 |
| ESK800-75RT | 2E3C:5745 | ohid | 待对照 |
| ESK900-75RT | 24AE:482A | ch585m | 待对照 |
| ESK900-83RT | 24AE:481B | ch585m | 待对照 |
| ESK900-98RT | 24AE:481A | ch585m | 待对照 |
| ESM612ACE | 24AE:464D | status-report7 | 待对照 |
| ESM612MAX | 24AE:4681 | status-report7 | 待对照 |
| ESM612PRO | 24AE:4631 | status-report7 | 待对照 |
| ESM622MAX | 24AE:4637 | status-report7 | 待对照 |
| ESM622PRO | 24AE:4632 | status-report7 | 待对照 |
| ESM652PRO | 24AE:4633 | status-report7 | 待对照 |
| ESM672MAX | 24AE:4640 | status-report7 | 待对照 |
| ESM672PRO | 24AE:4635 | status-report7 | 待对照 |
| ESPR5 | 24AE:1445 | status-report7-base | 待对照 |
| ESPR7 | 24AE:1458 | status-report7-base | 待对照 |
| ESPR7PRO | 24AE:1450 | status-report7-base | 待对照 |
| ESPR8 | 24AE:144E | status-report7-base | 待对照 |
| ESPR8PRO | 24AE:1441 | status-report7-base | 待对照 |
| MT550L | 24AE:1874 | telink-8273 · 无线 | 待对照 |
| MT550L | 24AE:4506 | telink-8273 · 有线 | 待对照 |
| MT560 | 24AE:1872 | telink-8273 · 无线 | 待对照 |
| MT560 | 24AE:4504 | telink-8273 · 有线 | 待对照 |
| MT750 PLUS | 24AE:1876 | telink-8273 · 无线 | 待对照 |
| MT750 PLUS | 24AE:4507 | telink-8273 · 有线 | 待对照 |
| MT760L | 24AE:1870 | telink-8273 · 无线 | 待对照 |
| MT760L | 24AE:4500 | telink-8273 · 有线 | 待对照 |
| MT760M | 24AE:1871 | telink-8273 · 无线 | 待对照 |
| MT760M | 24AE:4503 | telink-8273 · 有线 | 待对照 |
| MT770L | 24AE:1C00 | telink-8273 · 无线 | 待对照 |
| MT770L | 24AE:4C00 | telink-8273 · 有线 | 待对照 |
| P5 PLUS | 24AE:4827 | telink-base | 待对照 |
| P6 PLUS | 24AE:4825 | telink-base | 待对照 |
| V700diy-75 | 24AE:1907 | telink-keyboard · 无线 | 待对照 |
| V700diy-75 | 24AE:4807 | telink-keyboard · 有线 | 待对照 |
| V700diy-75 LRV | 24AE:6204 | telink-keyboard · 有线 | 待对照 |
| V700diy-75 LRV | 24AE:7204 | telink-keyboard · 无线 | 待对照 |
| V700diy-87 | 24AE:1906 | telink-keyboard · 无线 | 待对照 |
| V700diy-87 | 24AE:4806 | telink-keyboard · 有线 | 待对照 |
| V700diy-98 | 24AE:1905 | telink-keyboard · 无线 | 待对照 |
| V700diy-98 | 24AE:4805 | telink-keyboard · 有线 | 待对照 |
| V700diy-98 LRV | 24AE:1924 | telink-keyboard · 无线 | 待对照 |
| V700diy-98 LRV | 24AE:4824 | telink-keyboard · 有线 | 待对照 |
| VPR8 PLUS | 24AE:144D | status-report7-base | 待对照 |
| VPR8 RT MASTER | 24AE:145A | status-report7-base | 待对照 |
| VT0 | 24AE:1410 | status-report7 · 无线 | 待对照 |
| VT0 | 24AE:4610 | status-report7 · 有线 | 待对照 |
| VT0 Air MAX | 24AE:14A1 | status-report7 · 无线 | 待对照 |
| VT0 Air MAX | 24AE:46A1 | status-report7 · 有线 | 待对照 |
| VT0 MAX | 24AE:1420 | status-report7 · 无线 | 待对照 |
| VT0 MAX | 24AE:4620 | status-report7 · 有线 | 待对照 |
| VT0 MAX MASTER | 24AE:144A | status-report7 · 无线 | 待对照 |
| VT0 MAX MASTER | 24AE:464A | status-report7 · 有线 | 待对照 |
| VT1 | 24AE:1415 | status-report7 · 无线 | 待对照 |
| VT1 | 24AE:4615 | status-report7 · 有线 | 待对照 |
| VT1 MAX | 24AE:1444 | status-report7 · 无线 | 待对照 |
| VT1 MAX | 24AE:4644 | status-report7 · 有线 | 待对照 |
| VT1 SE | 24AE:1A11 | vt-ch584 · 无线 | 待对照 |
| VT1 SE | 24AE:4A11 | vt-ch584 · 有线 | 待对照 |
| VT1S MAX V2 | 24AE:1B0A | status-report7 · 无线 | 待对照 |
| VT1S MAX V2 | 24AE:4B0A | status-report7 · 有线 | 待对照 |
| VT1S V2 | 24AE:146A | status-report7 · 无线 | 待对照 |
| VT1S V2 | 24AE:466A | status-report7 · 有线 | 待对照 |
| VT2 | 24AE:1422 | status-report7 · 无线 | 待对照 |
| VT2 | 24AE:4622 | status-report7 · 有线 | 待对照 |
| VT2 Air MAX V2 | 24AE:1B12 | status-report7 · 无线 | 待对照 |
| VT2 Air MAX V2 | 24AE:4B12 | status-report7 · 有线 | 待对照 |
| VT2 MAX | 24AE:1423 | status-report7 · 无线 | 待对照 |
| VT2 MAX | 24AE:4623 | status-report7 · 有线 | 待对照 |
| VT2 MAX MASTER V2 | 24AE:1475 | status-report7 · 无线 | 待对照 |
| VT2 MAX MASTER V2 | 24AE:4675 | status-report7 · 有线 | 待对照 |
| VT2 MAX V2 | 24AE:1B09 | status-report7 · 无线 | 待对照 |
| VT2 MAX V2 | 24AE:4B09 | status-report7 · 有线 | 待对照 |
| VT2 RT MASTER V2 | 24AE:1B21 | lm20rt · 无线 | 待对照 |
| VT2 RT MASTER V2 | 24AE:4B21 | lm20rt · 有线 | 待对照 |
| VT2 RT V2 | 24AE:1B20 | lm20rt · 无线 | 待对照 |
| VT2 RT V2 | 24AE:4B20 | lm20rt · 有线 | 待对照 |
| VT2 ULTRA V2 | 24AE:1472 | telink-3228c · 无线 | 待对照 |
| VT2 ULTRA V2 | 24AE:4672 | telink-3228c · 有线 | 待对照 |
| VT2 V2 | 24AE:1468 | status-report7 · 无线 | 待对照 |
| VT2 V2 | 24AE:4668 | status-report7 · 有线 | 待对照 |
| VT2K MAX | 24AE:14A4 | status-report7 · 无线 | 待对照 |
| VT2K MAX | 24AE:46A4 | status-report7 · 有线 | 待对照 |
| VT2S Air MAX V2 | 24AE:1B13 | status-report7 · 无线 | 待对照 |
| VT2S Air MAX V2 | 24AE:4B13 | status-report7 · 有线 | 待对照 |
| VT2S MAX MASTER V2 | 24AE:1476 | status-report7 · 无线 | 待对照 |
| VT2S MAX MASTER V2 | 24AE:4676 | status-report7 · 有线 | 待对照 |
| VT2S MAX V2 | 24AE:1B04 | status-report7 · 无线 | 待对照 |
| VT2S MAX V2 | 24AE:4B04 | status-report7 · 有线 | 待对照 |
| VT2S V2 | 24AE:145D | status-report7 · 无线 | 待对照 |
| VT2S V2 | 24AE:465D | status-report7 · 有线 | 待对照 |
| VT3 | 24AE:1412 | status-report7 · 无线 | [rapoo-vt3-ii](../../../drivers/rapoo-vt3-ii) · 外设盒记录的 VT3 II 接收器 |
| VT3 | 24AE:4612 | status-report7 · 有线 | 待对照 |
| VT3 Air MAX V2 | 24AE:1B0C | status-report7 · 无线 | 待对照 |
| VT3 Air MAX V2 | 24AE:4B0C | status-report7 · 有线 | 待对照 |
| VT3 MAX | 24AE:1417 | status-report7 · 无线 | 待对照 |
| VT3 MAX | 24AE:4617 | status-report7 · 有线 | 待对照 |
| VT3 MAX MASTER V2 | 24AE:146D | status-report7 · 无线 | 待对照 |
| VT3 MAX MASTER V2 | 24AE:466D | status-report7 · 有线 | 待对照 |
| VT3 MAX V2 | 24AE:1B06 | status-report7 · 无线 | 待对照 |
| VT3 MAX V2 | 24AE:4B06 | status-report7 · 有线 | 待对照 |
| VT3 PRO V2 | 24AE:14A5 | status-report7 · 无线 | 待对照 |
| VT3 PRO V2 | 24AE:46A5 | status-report7 · 有线 | 待对照 |
| VT3 V2 | 24AE:1462 | status-report7 · 无线 | 待对照 |
| VT3 V2 | 24AE:4662 | status-report7 · 有线 | 待对照 |
| VT3S | 24AE:1411 | status-report7 · 无线 | 待对照 |
| VT3S | 24AE:4611 | status-report7 · 有线 | 待对照 |
| VT3S Air MAX V2 | 24AE:1B0D | status-report7 · 无线 | 待对照 |
| VT3S Air MAX V2 | 24AE:4B0D | status-report7 · 有线 | 待对照 |
| VT3S Air Max | 24AE:14A2 | status-report7 · 无线 | 待对照 |
| VT3S Air Max | 24AE:46A2 | status-report7 · 有线 | 待对照 |
| VT3S MAX | 24AE:1421 | status-report7 · 无线 | 待对照 |
| VT3S MAX | 24AE:4621 | status-report7 · 有线 | 待对照 |
| VT3S MAX MASTER | 24AE:144B | status-report7 · 无线 | 待对照 |
| VT3S MAX MASTER | 24AE:464B | status-report7 · 有线 | 待对照 |
| VT3S MAX MASTER V2 | 24AE:146E | status-report7 · 无线 | 待对照 |
| VT3S MAX MASTER V2 | 24AE:466E | status-report7 · 有线 | 待对照 |
| VT3S MAX V2 | 24AE:1B07 | status-report7 · 无线 | 待对照 |
| VT3S MAX V2 | 24AE:4B07 | status-report7 · 有线 | 待对照 |
| VT3S PRO V2 | 24AE:14A6 | status-report7 · 无线 | 待对照 |
| VT3S PRO V2 | 24AE:46A6 | status-report7 · 有线 | 待对照 |
| VT3S V2 | 24AE:1464 | status-report7 · 无线 | 待对照 |
| VT3S V2 | 24AE:4664 | status-report7 · 有线 | 待对照 |
| VT4 Air MAX V2 | 24AE:1B14 | status-report7 · 无线 | 待对照 |
| VT4 Air MAX V2 | 24AE:4B14 | status-report7 · 有线 | 待对照 |
| VT4 MAX MASTER V2 | 24AE:1477 | status-report7 · 无线 | 待对照 |
| VT4 MAX MASTER V2 | 24AE:4677 | status-report7 · 有线 | 待对照 |
| VT4 MAX V2 | 24AE:1B03 | status-report7 · 无线 | 待对照 |
| VT4 MAX V2 | 24AE:4B03 | status-report7 · 有线 | 待对照 |
| VT4 V2 | 24AE:145B | status-report7 · 无线 | 待对照 |
| VT4 V2 | 24AE:465B | status-report7 · 有线 | 待对照 |
| VT6 | 24AE:1451 | status-report7 · 无线 | 待对照 |
| VT6 | 24AE:4651 | status-report7 · 有线 | 待对照 |
| VT6 MAX | 24AE:1452 | status-report7 · 无线 | 待对照 |
| VT6 MAX | 24AE:4652 | status-report7 · 有线 | 待对照 |
| VT68 | 24AE:4832 | gcome-ap | 待对照 |
| VT7 | 24AE:1413 | status-report7 · 无线 | 待对照 |
| VT7 | 24AE:4613 | status-report7 · 有线 | 待对照 |
| VT7 Air MAX V2 | 24AE:1B0E | status-report7 · 无线 | 待对照 |
| VT7 Air MAX V2 | 24AE:4B0E | status-report7 · 有线 | 待对照 |
| VT7 MAX | 24AE:1416 | status-report7 · 无线 | 待对照 |
| VT7 MAX | 24AE:4616 | status-report7 · 有线 | 待对照 |
| VT7 MAX MASTER V2 | 24AE:1470 | status-report7 · 无线 | 待对照 |
| VT7 MAX MASTER V2 | 24AE:4670 | status-report7 · 有线 | 待对照 |
| VT7 MAX V2 | 24AE:1B05 | status-report7 · 无线 | 待对照 |
| VT7 MAX V2 | 24AE:4B05 | status-report7 · 有线 | 待对照 |
| VT7 PRO V2 | 24AE:14A7 | status-report7 · 无线 | 待对照 |
| VT7 PRO V2 | 24AE:46A7 | status-report7 · 有线 | 待对照 |
| VT7 SE | 24AE:1A00 | vt-ch584 · 无线 | 待对照 |
| VT7 SE | 24AE:4A00 | vt-ch584 · 有线 | 待对照 |
| VT7 V2 | 24AE:1460 | status-report7 · 无线 | 待对照 |
| VT7 V2 | 24AE:4660 | status-report7 · 有线 | 待对照 |
| VT7S Air MAX V2 | 24AE:1B0F | status-report7 · 无线 | 待对照 |
| VT7S Air MAX V2 | 24AE:4B0F | status-report7 · 有线 | 待对照 |
| VT7S MAX MASTER V2 | 24AE:1471 | status-report7 · 无线 | 待对照 |
| VT7S MAX MASTER V2 | 24AE:4671 | status-report7 · 有线 | 待对照 |
| VT7S MAX V2 | 24AE:1B02 | status-report7 · 无线 | 待对照 |
| VT7S MAX V2 | 24AE:4B02 | status-report7 · 有线 | 待对照 |
| VT7S PRO V2 | 24AE:14A8 | status-report7 · 无线 | 待对照 |
| VT7S PRO V2 | 24AE:46A8 | status-report7 · 有线 | 待对照 |
| VT7S V2 | 24AE:1459 | status-report7 · 无线 | 待对照 |
| VT7S V2 | 24AE:4659 | status-report7 · 有线 | 待对照 |
| VT8 Air MAX V2 | 24AE:1B15 | status-report7 · 无线 | 待对照 |
| VT8 Air MAX V2 | 24AE:4B15 | status-report7 · 有线 | 待对照 |
| VT8 MAX MASTER V2 | 24AE:1478 | status-report7 · 无线 | 待对照 |
| VT8 MAX MASTER V2 | 24AE:4678 | status-report7 · 有线 | 待对照 |
| VT8 MAX V2 | 24AE:1B0B | status-report7 · 无线 | 待对照 |
| VT8 MAX V2 | 24AE:4B0B | status-report7 · 有线 | 待对照 |
| VT8 V2 | 24AE:146C | status-report7 · 无线 | 待对照 |
| VT8 V2 | 24AE:466C | status-report7 · 有线 | 待对照 |
| VT9 | 24AE:1414 | status-report7 · 无线 | 待对照 |
| VT9 | 24AE:4614 | status-report7 · 有线 | 待对照 |
| VT9 Air MAX V2 | 24AE:1B10 | status-report7 · 无线 | 待对照 |
| VT9 Air MAX V2 | 24AE:4B10 | status-report7 · 有线 | 待对照 |
| VT9 MAX | 24AE:1418 | status-report7 · 无线 | 待对照 |
| VT9 MAX | 24AE:4618 | status-report7 · 有线 | 待对照 |
| VT9 MAX MASTER V2 | 24AE:1473 | status-report7 · 无线 | 待对照 |
| VT9 MAX MASTER V2 | 24AE:4673 | status-report7 · 有线 | 待对照 |
| VT9 MAX V2 | 24AE:1B08 | status-report7 · 无线 | 待对照 |
| VT9 MAX V2 | 24AE:4B08 | status-report7 · 有线 | 待对照 |
| VT9 PRO V2 | 24AE:14A9 | status-report7 · 无线 | 待对照 |
| VT9 PRO V2 | 24AE:46A9 | status-report7 · 有线 | 待对照 |
| VT9 V2 | 24AE:1466 | status-report7 · 无线 | 待对照 |
| VT9 V2 | 24AE:4666 | status-report7 · 有线 | 待对照 |
| VT9S Air MAX V2 | 24AE:1B11 | status-report7 · 无线 | 待对照 |
| VT9S Air MAX V2 | 24AE:4B11 | status-report7 · 有线 | 待对照 |
| VT9S MAX MASTER V2 | 24AE:1474 | status-report7 · 无线 | 待对照 |
| VT9S MAX MASTER V2 | 24AE:4674 | status-report7 · 有线 | 待对照 |
| VT9S MAX V2 | 24AE:1B01 | status-report7 · 无线 | 待对照 |
| VT9S MAX V2 | 24AE:4B01 | status-report7 · 有线 | 待对照 |
| VT9S PRO V2 | 24AE:14AA | status-report7 · 无线 | 待对照 |
| VT9S PRO V2 | 24AE:46AA | status-report7 · 有线 | 待对照 |
| VT9S V2 | 24AE:1453 | status-report7 · 无线 | 待对照 |
| VT9S V2 | 24AE:4653 | status-report7 · 有线 | 待对照 |
