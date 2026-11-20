# 迈从通用 8K 接收器与板载设置

在预览商店按完整型号安装插件，即确认允许外设盒查询该接收器。日常刷新通过 `0x0900` 读取配对鼠标的内部 VID/PID、电量、充电标志和连接状态。运行时同时核对各型号声明的接收器（`3837:1018` 或 K5 V2 的 `3837:101C`）、HID `FF01:0001`、64 字节输入和输出，以及回复中该款鼠标的配对身份。休眠短应答、0% 占位或配对其他型号时，不提供电量。

打开支持指针参数读取的设备概览或点击“读取指针参数”时，外设盒发送额外的只读查询。普通电量刷新只顺带接收等待窗口里的档位事件。进入详情后读取性能、DPI 和型号提供的抬升距离。后台电量刷新保持独立。

| 读数 | 取值方式和含义 |
| --- | --- |
| 连接与充电 | 连接值 1 表示有线，2 表示 2.4 GHz，其余未知。充电值 0 表示未充，1 表示正在充，2 表示已满。未知充电值保持空。 |
| 配置回报率 | `0x0002` 返回有线和无线两个档位索引。界面只在二者一致、官方型号加接收器的表中存在该档位时显示换算所得的 Hz。两个索引均随有效应答返回。没有实测吞吐率。 |
| X/Y 轴 DPI 配置 | `0x0003` 读取性能回复给出的板载配置号。性能请求缺失时仍单独读取配置 0。回复须回显所请求的配置及方向，提供六个小端槽位，并声明 1 到 6 个有效档位。X 应答声明支持独立 Y 轴时再读 Y。各轴只显示已启用且数值有效的档位。界面区分该配置的存储档位和设备主动上报的档位。 |
| 当次上报的活动档位 | 收到 `0x0600` 的档位事件且索引为 0 到 5 时，返回该索引。设备详情按第 1 到第 6 档显示。 |

率或轴配置读取失败时，已经取得的电量、另一项配置和可解析的索引仍可用。此处的设备匹配范围为各型号目录所列的 8K 接收器。其他接收器与直连模式有各自的设备目录。

## 原生设置（0.4.13-preview.43）

在本地设置中选择一项并应用，驱动会保存本次涉及的原值、发送写入、再从同一设备读回。失败时尝试恢复，并分别告知已读回原值或恢复未确认。官网程序及图片并未打包进插件。型号上限、设置位、命令和页面取值参考了 2026 年 11 月 20 日的 [M HUB 官方前端](https://www.mchose.com.cn/assets/main-DIt5vAcV.js)（SHA‑256 `cb10c9d3d29505bef2e2d3207dbe223601ff84c46d772b4a0f1e93ef02391c1b`）。

| 能力 | 本地读写 |
| --- | --- |
| DPI | 编辑独立 X/Y 数值、切换档位、调整 1–6 档、切换双轴分设；启用双轴从 X 复制并可单独调整。单档 `0x0104`，布局 `0x0103`，读回 `0x0003`。按型号范围 200–26000 / 42000 / 50000。 |
| 回报率 | 根据鼠标和接收器组合的表编码 Hz，同时设置有线与无线配置，保留 DPI 等其他位。`0x0102` 写入，`0x0002` 读回。 |
| 性能 | 左右按键消抖 1–20 ms、休眠 0–100 分钟（0 为永不）、3 个性能模式、传感器旋转、Motion Sync、直线修正、纹波修正。读取完整配置后只修改涉及的字段，并保留原报文的保留位。 |
| 抬升与玻璃表面 | 按型号提供 2、3 或 5 个抬升距离。3955 系使用 `0x0009` / `0x0109`，其他型号修改传感器位。允许玻璃模式的型号启用时同步调到官网指定的抬升档位；两项一起读回及恢复。 |
| 板载配置切换 | 使用设备声明的槽位数与官方编辑器的 3 个配置范围，`0x0906` 切换后再读 `0x0002` 核对。写入参数可以携带原配置号，设备已切换时提示重新打开。 |

设备寄存器格式来自官网公开运行时的 `mcSetCommand`、`allInfoToArr`、`dpiToArr`、`setSingleDpi`、`mouseDiffConfig`、`newMouse3955` 与对应产品页的事件处理。本项目独立实现 Rust 协议与页面，不含厂商代码。以上为公开程序支持、本项目样本和读回逻辑检查，本次无对应实体设备写入验证。

## 板载改键、宏和配置名称（0.4.13-preview.44）

读取当前配置上的鼠标左右键、中键、前进、后退与设备提供的 DPI 键。用户可选择鼠标功能、键盘键及组合键、多媒体、DPI 循环或禁用。通过 `0x0001` 读取实际键表，用 `0x0101` 仅写指定按键并重新读取；其他键和未知编码保留原值。

板载宏支持按一次、按住重复、再次按下停止、按其他键停止。支持键盘按下与松开、鼠标键、双向滚轮和毫秒延时。官网编辑器上限为 1008 个事件，按设备实际声明缩小。软件使用 `0x090c` 分页读取、`0x090b` 分页写入，保存后逐字节读回，最后再绑定按钮。当前按键独占的可识别宏可以原位编辑；其他配置共用、内容未知或未绑定却含数据的位置均会保留。一个操作中途失败时，分别报告按键和宏存储内容的恢复结果。

板载配置名通过 `0x0905` 分块读取、`0x0910` 分块写入，最多 60 个 UTF-8 字节。官方或云端模板的关联后缀在改名时原样保留，界面可用名称长度随之调整。用户自建宏名最多 54 个 UTF-8 字节。

命令来源为上文迈从官方网页中的 `readMouseKeyInfo`、`setMouseKeyInfo`、`getMacroPacketArr`、`setOnePackMacro`、`readMacroName` 与 `setBoardName`。模式编号与界面标注采用同版本[官方中文资源](https://www.mchose.com.cn/assets/index-BLfQ5SSo.js)。本项目用包含不同长度键位的样本，以及四类事件的编码/解码检查此实现。该批未进行实机写入验证。

## 图形改键与 DPI 加减档（0.4.13-preview.50）

在「设备调节 → 按键」选择图上的按键，可绑定「DPI 加一档」和「DPI 减一档」。两个动作直接存在当前板载配置，沿用原有的单键写入及读回。原厂 `dpiMap` 和 `mouseKeyToArr` 将两种动作分别编码为 `01 02 00 00`、`01 03 00 00`。按键槽依次为左键、右键、中键、前进和后退。只有原厂 `supportDpiKeyDevice` 包含该型号、且当前键表有有效第六条时，才可选择第六个 DPI 键。A7 Magnetic、R7 Ultra 和 K5 V2 各有五个可编辑键，仍然可以把其中任一键改成 DPI 加减档，并保留至少一个左键。

| 8K 接收器配对型号 | 原生图与原厂对照 |
| --- | --- |
| A5 V3 Pro、Ultra、Ultra+(3955) | 共用 A5 V3 轮廓、滚轮及侧键位置。[Pro 说明书](https://cdn.mchose.com.cn/configCenter/static/documents/A5_V3_Pro_l1xmekjb0ua82pvg.pdf) · [Ultra 系列说明书](https://cdn.mchose.com.cn/configCenter/static/documents/A5_V3_Ultra_Plus_o7dm7pggxqwyewvm.pdf) |
| A7 V3、Pro、Pro+、Ultra；A7 V3-X | A7 V3 常规外壳与 V3-X 的正视图分别绘制。[标准版说明书](https://cdn.mchose.com.cn/configCenter/static/documents/A7_V3_29s1a18ylqgt4p0b.pdf) · [Plus 系列说明书](https://cdn.mchose.com.cn/configCenter/static/documents/A7_V3_Plus_842di96d5avkxyg8.pdf) · [V3-X 说明书](https://cdn.mchose.com.cn/configCenter/static/documents/A7_V3_X_fcqd6lk7ecyzbz55.pdf) |
| A7 V3 Magnetic | 原厂页面未提供此分支的单独外形图，原生示意图单独画出左右键、滚轮和前后侧键。 |
| K5 Pro、Ultra | 按各自的正视图提供可点击的左右键、滚轮和侧键；第六键通过标签选择。[K5 系列说明书](https://cdn.mchose.com.cn/configCenter/static/documents/K5_9tso4iayd4sxc6dx.pdf) |
| K5 V2（101C） | 单独核对 4040 配对身份，以协议五键图改键。[独立插件及型号参数](../../drivers/mchose-k5-v2/UPSTREAM.md) |
| K7 V2 Pro、Pro+、Ultra、Ultra+ | 四款共用原厂 K7 V2 说明书，原生图保留平直的前缘及侧键位置。[K7 V2 说明书](https://cdn.mchose.com.cn/configCenter/static/documents/K7_V2_p6zlvnpdkd4jw8de.pdf) |
| R7 Ultra | 使用右手工学外形，滚轮前的小开关未列入原厂可改键槽。[R7 Ultra 说明书](https://cdn.mchose.com.cn/configCenter/static/documents/R7_Ultra_ycuifz48i17ntgcs.pdf) |

当前包可分别选择同型号的多个接收器。每次请求先定位选定接口，再核对 `0x0900` 配对身份。本项目已检查动作字节、有效键数及各图的命中区域；本批 17 款没有逐款实机写入报告。[项目原创示意图](diagrams) · [K5 Ultra 既有示意图](../../drivers/mchose-k5-ultra/button-diagram.svg)

## 依据与验证

Vigil Watt 的物理样本在该接收器上自报 `3837:4026`、100% 电量、充电标志 0 和连接方式 2。上游现场记作 K7 V2 Ultra+，官方表将此配对编号分配给 A5 V3 Ultra+，与另一个本仓库既有 A7 V3 Ultra+ 样本也有冲突。本批绕开 `4026`。K7 V2 Ultra+ 插件只接受其官方编号 `4028`。逐款对照 16 个鼠标内部 PID 以及官方配置里的 `1018` 接收器槽位后，范围包括 A5 V3 三款、A7 V3 六款、K5 两款、K7 V2 四款及 R7 Ultra。原有 A7 V3 Ultra+ 插件继续按其独立取证条件工作。

上游已经解析 DPI 配置及换算回报率，并给出合成应答测试。上述 16 个型号的这些功能尚无逐款物理验证。本项目使用上游接收器抓包和合成应答测试解析、配对隔离和配置规则，尚未进行实机测试。

- [接收器抓包、命令及现场记录](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/brands/mchose.md)
- [厂商前端和型号/接收器的交叉对照](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/docs/hardware/reference/mchose-dpi-report-rate-audit.md)
- [官方表整理版](../../catalog/upstream/vigil-watt/mchose-catalog.json)
- [上游状态和指针参数查询](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/MchosePlugin.cs)
- [上游编解码](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/Mchose0x4DCodec.cs)
- [回报率换算条件](https://github.com/evren-oct/vigil-watt/blob/6315988f54e47fc030ae4c535dc64ae27c08f471/src/VigilWatt.Core/Plugins/Mchose/MchoseRatePolicy.cs)

接收器适配以 Rust 独立实现。上游数据与抓包的 MIT 许可随每个包提供。[许可证](VIGIL-WATT-MIT.txt)
