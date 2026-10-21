# 后续设备接入任务队列

核查日期：2026-10-21。适用基线：Peripheral Desk Drivers 稳定版 0.4.2、Logitech G305 `preview.1`。这是一份按先后顺序执行的候选队列，不代表已支持、已测试或承诺完成时间。当前四款稳定设备的维护与 G305 预览继续独立推进。

## 来源、复用与准入规则

本轮可复现的 OpenMouse 源码固定点是 [`mouse-protocol` `bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc)，许可证为 [`LICENSE` 中的 GNU AGPL v3](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/LICENSE)。它提供不同品牌的协议编解码和 WebHID 驱动，能给出报文、命令、型号表和验证线索；OpenMouse 标注 `verified` 或报告真实抓包，仍不等于本项目在 Windows 插件和目标固件上的实测。若上游换版本，先固定新 commit、复查逐文件许可证及证据，再决定是否借用。本项目为 AGPL-3.0-only，OpenMouse 源码可在满足其 AGPL-3.0 条款、保留版权/许可证和修改声明、提供对应源码等义务后复用；具体依赖/文件和分发方式应记录在包许可清单中。仅凭协议事实另写实现也必须保留来源与具体型号依据。

每个型号单独核对：精确 VID/PID、接口/usage page、报告方向与长度，以及运行时能否验证设备身份。VID、PID、同接收器、同 ODM、同品牌或型号名相似，都不能单独作为兼容依据。有精确且可追溯的上游型号身份与协议实现时，可以发布明确标注“未实机验证”的只读 preview；真机验证是升为已验证/稳定支持的依据，不是所有 preview 的前置条件。协议或型号身份证据不可靠、相互冲突，或本机枚举不符合已声明范围时，停止出包或拒绝该设备。任何身份字段、校验和、回复形状或状态读取不符，立即停止该候选，不扩大匹配范围。

## 有序任务

### 0. 当轮并行：Razer Basilisk V3 Pro 有线只读预览

- **状态 / 优先级：** 本轮已实现独立只读插件 `razer-basilisk-v3-pro-wired`；软件检查与公开包以对应 release 为准。未实机验证，该精确型号及连接路径不重复排入后续实现队列。
- **目标 / 身份：** Razer Basilisk V3 Pro 有线，USB `1532:00AA`；仅唯一 `MI_00`、91 字节 Feature 集合。该集合条件依据固定协议推导而非本机描述符实测；不扩展到 `00AB` 接收器、35K 版本或其他 Razer 型号。
- **依据 / 许可：** OpenRazer 固定 [`477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d` 的 `mouse.py` Basilisk V3 Pro Wired 类](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py#L1694) 与 [`razermouse_driver.h` 的 00AA 定义](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L92) 明确对应 1532:00AA。类列出电量/充电读取方法；本包仍只暴露只读能力。OpenRazer 为 GPL-2.0-or-later，若实际复用其代码，须单独确认该代码与本包 AGPL 发行组合的兼容条件并履行许可证义务。
- **接口产物：** 独立设备 DLL、精确 metadata/manifest、只读电量与充电状态；预览 catalog 条目。沿用 ABI 1 的 `read-state`，不声明设置或灯光能力。
- **验收：** 精确 HID 过滤；读命令/报告长度与回复校验均有固定来源；未知型号拒绝连接；模拟或抓包覆盖充电、放电、无回复和格式错误。可以在无真机时发布标注“未实机验证”的只读 preview；真机到手后才记录 hardware-verified。预览包保持 0.4.x ABI 兼容。
- **阻碍 / 退出条件：** 缺少固定源码支持的精确型号身份、配置集合或可靠只读事务证据时，不发布包；没有真机本身不阻止标注为未验证的只读 preview。本项保留为已实现记录，后续只跟踪具体固件的实机验证。

### 1. MCHOSE A7 V3 Ultra+ 接收器只读插件

- **状态 / 优先级：** 已实现 0.4.4-preview.1 只读插件，等待对应发布；公开实现里读取路径的硬件证据较强，且与现有初代 A5 协议明显不同。
- **目标 / 身份：** MCHOSE A7 V3 Ultra+，优先接收器路径 `3837:1018`（8K 接收器）；有线 `3837:4033` 暂不纳入。需精确 `0xff01:0001` 配置集合；上游 WebHID 以 report ID `0x4D` 单独传入、63 字节 payload 作为报告体，Windows HID 缓冲区总长为 64 字节（含 report ID），输出请求、输入回复；并核对设备 product string。`0x0900` 回报的内部 `productId=0x4026` 在上游表里对应 A5 V3 Ultra+，因此不能把它单独当成型号身份；字符串缺失或不一致就拒绝型号级支持。
- **协议证据 / 源码：** 固定来源 [`src/mchose/v3.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/v3.ts)、[`src/drivers/mchose/v3-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/v3-hid.ts)、[`docs/mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md)。文档记录真实 A7 V3 Ultra+ 8K 接收器上的 `0x0900` 身份/电量、`0x0002` 设置、`0x0003` DPI 和 `0x0001` 按键读取；**上游说明写入尚未实机验证**。本阶段只实现 `0x0900` 只读电池与连接状态，不读宏、不发 `0x01xx` 或其他写命令。由于固定上游文档给出该精确接收器路径的真实读取记录，允许无本机真机时发布明确标注未验证的只读 preview；若运行时产品字符串缺失或冲突则拒绝型号级声明。
- **接口产物：** `mchose-a7-v3-ultra-plus` 单独设备包及 `read-state` 适配器；返回电量、充电/连接状态、固件未知时留空。manifest 只列接收器连接和只读能力。
- **验收：** PID、usage、报告长、报告 ID、命令与校验严格匹配；`0x0900` 内 vendor id 为 MCHOSE、产品字符串精确匹配时才报告型号；失联/设备休眠、错误产品字符串、无校验回复、未知产品号都应安全失败；对真实目标鼠标完成多轮电量及连接状态对照后才升验证等级。
- **阻碍 / 退出条件：** 主机只暴露共享接收器名且无法用鼠标字符串或其他可信运行时信息验证精确型号时，只能交付不安装的探测记录。写入、改键、宏、底座 RGB 均不属于本项。

### 2. VGN Dragonfly F2 Master+ 未验证只读预览

- **状态 / 优先级：** 已实现 0.4.4-preview.1 只读插件，等待对应发布；固定上游驱动精确声明 F2 Master+ 的接收器/有线 PID 和报告形状，可作为未实机验证只读 preview 的来源依据；preview 范围严格限于已公开读取。
- **目标 / 身份：** VGN Dragonfly F2 Master+；无线接收器 `3554:FB56`，有线 `3554:FB57`。需 `0xff02` 集合、report ID `0x08`。OpenMouse WebHID 读写 16 字节 payload，Windows HID 报告缓冲区通常为 17 字节（1 字节 report ID + 16 字节 payload）；移植时分别核对报告描述符和 API 缓冲区语义。F1 Pro (`F502/F503`) 和 F2 Pro Max (`FB3D/FB3E`) 不是此候选，也不因 VID 或所谓共用方案被纳入。
- **协议证据 / 源码：** 固定来源 [`src/vgn/index.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/vgn/index.ts)、[`src/drivers/vgn/hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/hid.ts)、[`src/drivers/vgn/protocol.test.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/protocol.test.ts)。代码含电量命令 `0x04`、校验和、16 字节传输与读配置实现；该驱动还含写设置和借用 Teevolution 的按键映射，均不纳入本只读项。上游单元测试的伪设备/捕获帧不证明某个本机固件已验证。
- **接口产物：** 固定上游驱动明确将这对 VID/PID、报告形状绑定到 Dragonfly F2 Master+，可据此先发布标注未实机验证的 `vgn-f2-master-plus` 只读 preview DLL/catalog 项，范围限电量、充电状态和固件版本；若运行时身份信息与该声明冲突，则拒绝连接。
- **验收：** 匹配 VID/PID、usage、report ID/长度；帧校验和、命令回显、异常/超时测试通过；不得仅靠 3554 或产品 PID 宣告 F2 Master+。固定来源与报告/校验验证通过，preview 对配置存储无写操作；有真机后再对照电量和充电状态并更新验证等级。
- **阻碍 / 退出条件：** 若 USB 身份、report 形状或公开实现无法把该 transport 可靠绑定到 F2 Master+，才停止出包；没有本机真机不阻止带未验证标记的只读 preview。后续 F1/F2 家族分别建候选，不共享身份表。

### 3. ATK F1 Ultimate 身份冲突排查后再做只读 preview

- **状态 / 优先级：** P3；协议通道有公开实现，但现有资料对这个名字对应的 USB ID 存在冲突，先解冲突再碰实际设备。
- **目标 / 身份：** 上游固定目录中 `ATK F1 Ultimate` 对应 `373B:102E`、CID/MID `2,42`、usage `0xFF02:0002`；同目录不是实测保证。现有项目品牌页/研究记录另列过 `373B:1031`，在实物/原始枚举证据解决差异前，两个 ID 都不允许新包认领。需 report ID `0x08`。OpenMouse WebHID 使用 16 字节 payload，Windows HID 报告缓冲区通常为 17 字节（含 report ID）；运行时命令 `0x10` 的 CID/MID 响应 `2,42` 验明身份，之后才读取电量命令 `0x04`。有线与接收器模式分别核对，不因一个模式通过推广到另一模式。
- **协议证据 / 源码：** 固定来源 [`src/drivers/atk/products.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/atk/products.ts)、[`src/drivers/atk/device-catalog.generated.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/atk/device-catalog.generated.ts)、[`src/drivers/atk/hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/atk/hid.ts)。源码实现了 CID/MID 解歧、版本/电量/EEPROM 读取与大量设置写入；只复核 `0x10` 和 `0x04` 读取，不调用任何写操作。不能误套 `373B:1216` 的 `FF05` 协议，也不能按同品牌其他 ATK 的 CID/MID 认领。
- **接口产物：** 先交一份枚举/应答证据记录；只有 `373B:102E` 与现有记录中的 `373B:1031` 冲突经权威型号资料或实机枚举解决后，才单独生成 `atk-f1-ultimate` 电量只读 preview 插件，manifest 精确限定确认后的 transport。若冲突无法解决则停止该型号包。
- **验收：** 记录两台设备并排枚举差异、每个 HID 集合的 report ID/方向/长度、CID/MID 原始回复；确认 102E/1031 差异属于哪一具体产品/接收器；重复读取电量且无写帧。包装器对无法确认的 CID/MID 或另一 PID 返回“不支持”。
- **阻碍 / 退出条件：** PID 冲突未由硬件和可追溯资料解决前不出包；CID/MID 不是 `2,42`、报文不是预期长度/集合或无法区分有线/接收器时停止，不扩大到 F1 系列。

### 4. 只读验证通过后，才开启该精确设备的设置写入

- **状态 / 优先级：** P4；依赖任务 1–3 中某个确切型号先达到本机只读验证。首选先在已经完成读取实测的 MCHOSE A7 V3 Ultra+ 上单项评估，不同时开放多个品牌。
- **型号 / 传输 / 身份：** 只使用前置任务已经实测通过的同一型号、同一 HID 集合和连接路径；不以同 VID/PID、同代际或同接收器推导其他设备。
- **协议证据 / 源码：** A7 V3 写入路径可在上述固定 rev 的 `src/mchose/v3.ts`、`src/drivers/mchose/v3-hid.ts` 找到；文件注释明确编码器/写入没有在硬件上发送过。它是研究线索，不是写入许可或安全证明。
- **接口产物：** 每次只实现一个可回读的设置动作，通过 ABI `apply-setting` 暴露；先完成安全备份格式、输入限值、读-改-写-回读与失败恢复，再加入 manifest capability。读写代码独立于宏/灯效。
- **验收：** 同一实机先留存完整原状态；逐项一次只改一个字段；读回与厂商工具一致；恢复原值成功；拔接收器/掉电/超时后旧值不被零值覆盖；不认识的字段原样保留。未经恢复验证的写入维持隐藏/关闭。
- **阻碍 / 退出条件：** 写入回读不稳定、无法完整读取被整块覆盖的数据、回滚失败或固件版本不明时，禁止发布该写能力。VGN 与 ATK 需单独重复此流程，不沿用 A7 结论。

### 5. 宏数据：只在完整备份与可恢复闭环成立后启动

- **状态 / 优先级：** P5，当前未达到开发准入条件；不承诺会形成可写宏编辑器。
- **候选型号：** 现有 `mchose-a5-ultra` 包所支持的初代 A5 精确设备及其当前验证 transport；不转移到 A5 V2、A7 或 A5 V3。现有品牌页记录：宏页不能完整备份，因此当前拒写。
- **来源 / 复用边界：** OpenMouse 固定源码的 `src/drivers/mchose/a5-gen1-hid.ts` 是另一套 `0x2023` CompX A5 Pro Max 协议，不能证明现有 `3837:100C` 初代 A5 Ultra 的宏格式。A7 V3 的 `readMacro` 资料也不能跨型号复用。
- **接口产物：** 只有对现有 A5 Ultra 真机确认完整页布局、校验、分页、长度和空宏含义后，才实现单独的无损读取/导出；写入接口须另设阶段，不随读取上线。
- **验收：** 连续两次完整备份逐字节一致；可把原始备份恢复到同一设备并校验；损坏、短读、未知字段、断连时不发写命令；原厂宏与名称可无损往返。只有此闭环才提议打开编辑能力。
- **阻碍 / 退出条件：** 任一页不能完整获取，继续拒绝写入，不展示可编辑状态。任何源文件若没有明确许可证则不复制；本候选不依赖 OpenMouse 其他 A5 代际的相似报文。

### 6. 照明：独立的 MCHOSE MagDock 插件，排在设备设置和宏之后

- **状态 / 优先级：** P6；与鼠标状态分开建项，灯在底座而非 A7 V2 鼠标本体。
- **目标 / 身份：** MCHOSE MagDock 充电底座，`3837:1012`、usage `0xFF00`；先用 HID 产品名与集合确认是底座。不得把效果发送给同 VID 的鼠标配置集合。
- **协议证据 / 源码：** 固定来源 [`src/mchose/dock.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/dock.ts)、[`src/drivers/mchose/dock-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/dock-hid.ts)、[`docs/mchose-protocol.md` MagDock 部分](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md)。协议记录有真实底座效果抓包与读回代码；驱动含灯光写入，仍须在目标底座验证，并做 AGPL 来源审查。
- **接口产物：** 独立 `mchose-mag-dock` 插件；若宿主元数据/ABI 尚不能给非鼠标设备提供 `lighting-read` / `lighting-apply` 能力，先提交能力扩展设计，不能伪装为鼠标 `read-state`。
- **验收：** 读出原模式/颜色/亮度，逐项写入效果并回读，再恢复原值；坏回复、断连和异常模式不触发全零重置；catalog 显示为充电底座，非鼠标 DPI/性能页。
- **阻碍 / 退出条件：** 未取得底座实物、读取不能稳定恢复或宿主能力不匹配时只保留协议调查，不发安装包；不影响既有 G2 Pro 灯效包。

## 本轮选择结论

快速开源接入的可执行路径是：当轮 Basilisk 有线、MCHOSE A7 V3 Ultra+ 接收器和 VGN F2 Master+ 均可在精确公开身份/协议依据下发布清楚标注未实机验证的只读 preview；MCHOSE 必须以设备 product string 化解 `0x0900` 产品号与型号名不一致，VGN 只限固定上游明确绑定的 `FB56/FB57` transport。ATK F1 Ultimate 因 `102E` 与 `1031` 冲突，先排查身份，未解决则不出包。真机用于升级验证等级；设置回写、宏与灯效仍需各自对应的实机恢复验证，按队列后续执行。



