# 按厂商跟进 Vigil Watt

在[型号、接收器和插件对照表](COVERAGE.md)中搜索鼠标、键盘或手柄的型号。点插件名可查看其支持范围。对照表也列出各厂商协议从上游稳定版到开发版的改动，以及目前只有厂商登记的身份。要安装插件，请在外设盒的驱动商店按型号搜索。[外设盒厂商目录](../../README.md)

| 厂商 | 查阅内容 |
| --- | --- |
| 迈从 | [设备身份、自报型号、共用接收器和回报率表](mchose-catalog.json) · [协议规格](mchose.protocols.json) |
| 雷柏 | [型号、有线和接收器身份](rapoo-catalog.json) · [协议规格](rapoo.protocols.json) |
| 北通 | [型号、官方 PID 和验证记录](betop-catalog.json) · [协议规格](betop.protocols.json) |
| 罗技 | [Unifying、Bolt、LIGHTSPEED 等接收器系列、槽位和电压曲线](logitech-hidpp.json) |

## 同步上游

在仓库根目录运行以下命令，需要 Python 3.11 或更新版和 Git。首次联网对比会在 `work/upstreams/vigil-watt` 建立参考仓库。

```shell
python scripts/sync_vigil_watt.py --vendor Mchose
python scripts/sync_vigil_watt.py --write
```

第一条只列出上次同步后迈从型号、接收器、协议和源码文件的变化。第二条将整批资料固定到同一个开发提交，根据 GitHub 最新稳定 Release 对照所有厂商，更新来源清单和可浏览的型号表。可以先省略所有参数预览完整变化。`--check` 在来源或对照表需要更新时返回 1。

已有上游 Git 仓库时，可用本地的完整 Git 提交或分支回放某一批变动。以下命令在已有相应 Git 对象的情况下可离线使用。

```shell
python scripts/sync_vigil_watt.py --checkout ../vigil-watt --stable-tag v1.3.1 --dev-ref 6315988f54e47fc030ae4c535dc64ae27c08f471 --check
```

确认某条上游身份与已实现插件的型号、接口和连接方式对应后，在 [plugin-links.json](plugin-links.json) 填入现有插件 ID 和适用连接方式，再重新生成对照表。可以直接添加共享协议的已实现功能，在插件记录中注明实物或公开实现的依据。同步命令只更新本仓库的研究资料，插件按原有流程签名发布。

[来源与固定提交](sources.json) · [上游 MIT 许可证](LICENSE) · [贡献指南](../../../CONTRIBUTING.md#vendor-research)
