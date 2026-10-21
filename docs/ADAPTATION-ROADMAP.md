# 后续设备接入任务队列

核查日期：2026-10-21。适用基线：Peripheral Desk Drivers 稳定版 0.4.2、Logitech G305 `preview.1`。这是一份按先后顺序执行的候选队列，不代表已支持、已测试或承诺完成时间。当前四款稳定设备的维护与 G305 预览继续独立推进。

## 来源、复用与准入规则

本轮可复现的 OpenMouse 源码固定点是 [`mouse-protocol` `bac031567b28f2a55c616fc525b03f2d5fa452cc`](https://github.com/OpenMouse-Project/mouse-protocol/tree/bac031567b28f2a55c616fc525b03f2d5fa452cc)，许可证为 [`LICENSE` 中的 GNU AGPL v3](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/LICENSE)。它提供不同品牌的协议编解码和 WebHID 驱动，能给出报文、命令、型号表和验证线索；OpenMouse 标注 `verified` 或报告真实抓包，仍不等于本项目在 Windows 插件和目标固件上的实测。若上游换版本，先固定新 commit、复查逐文件许可证及证据，再决定是否借用。本项目为 AGPL-3.0-only，OpenMouse 源码可在满足其 AGPL-3.0 条款、保留版权/许可证和修改声明、提供对应源码等义务后复用；具体依赖/文件和分发方式应记录在包许可清单中。仅凭协议事实另写实现也必须保留来源与具体型号依据。

按 [`UPSTREAM-REUSE.md`](UPSTREAM-REUSE.md) 执行证据分级与移植验收。对每个精确型号、连接路径和有证据的固件范围，逐项记录上游实际展示的能力；精确 VID/PID、接口/usage、报告形状和运行时身份仍用于限定适用范围。只有型号名相似、共享 VID/PID 或家族共用协议不足以扩展支持。上游针对同一设备与连接路径记录的真实抓包/硬件验证，可以作为该项能力的协议依据；本项目的移植检查验证 Windows HID 映射、ABI/包行为及安装或升级时用户数据保留，不重复跑一遍已被上游验证的协议流程。上游未实测的操作、未识别固件差异及身份冲突仍不得写成已验证能力；可以继续开发协议适配或模拟覆盖，但发布能力须有该操作与精确目标相符的可追溯证据，并明确标注本项目的硬件验证状态。没有本机设备本身不构成禁止开发或发布的条件。

## 有序任务

### 0. 当轮并行：Razer Basilisk V3 Pro 有线只读预览

- **状态 / 优先级：** 本轮已实现独立只读插件 `razer-basilisk-v3-pro-wired`；软件检查与公开包以对应 release 为准。未实机验证，该精确型号及连接路径不重复排入后续实现队列。
- **目标 / 身份：** Razer Basilisk V3 Pro 有线，USB `1532:00AA`；仅唯一 `MI_00`、91 字节 Feature 集合。该集合条件依据固定协议推导而非本机描述符实测；不扩展到 `00AB` 接收器、35K 版本或其他 Razer 型号。
- **依据 / 许可：** OpenRazer 固定 [`477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d` 的 `mouse.py` Basilisk V3 Pro Wired 类](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py#L1694) 与 [`razermouse_driver.h` 的 00AA 定义](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.h#L92) 明确对应 1532:00AA。类列出电量/充电读取方法；本包仍只暴露只读能力。OpenRazer 为 GPL-2.0-or-later，若实际复用其代码，须单独确认该代码与本包 AGPL 发行组合的兼容条件并履行许可证义务。
- **接口产物：** 独立设备 DLL、精确 metadata/manifest、只读电量与充电状态；预览 catalog 条目。沿用 ABI 1 的 `read-state`，不声明设置或灯光能力。
- **验收：** 精确 HID 过滤；读命令/报告长度与回复校验均有固定来源；未知型号拒绝连接；模拟或抓包覆盖充电、放电、无回复和格式错误。可以在无真机时发布标注“未实机验证”的只读 preview；真机到手后才记录 hardware-verified。预览包保持 0.4.x ABI 兼容。
- **阻碍 / 退出条件：** 缺少固定源码支持的精确型号身份、配置集合或可靠只读事务证据时，不发布包；没有真机本身不阻止标注为未验证的只读 preview。本项保留为已实现记录，后续只跟踪具体固件的实机验证。

### 1. MCHOSE A7 V3 Ultra+ 接收器只读插件

- **状态 / 优先级：** 已发布 0.4.4-preview.2 只读插件；公开实现里读取路径的硬件证据较强，且与现有初代 A5 协议明显不同。
- **目标 / 身份：** MCHOSE A7 V3 Ultra+，优先接收器路径 `3837:1018`（8K 接收器）；有线 `3837:4033` 暂不纳入。需精确 `0xff01:0001` 配置集合；上游 WebHID 以 report ID `0x4D` 单独传入、63 字节 payload 作为报告体，Windows HID 缓冲区总长为 64 字节（含 report ID），输出请求、输入回复；并核对设备 product string。`0x0900` 回报的内部 `productId=0x4026` 在上游表里对应 A5 V3 Ultra+，因此不能把它单独当成型号身份；字符串缺失或不一致就拒绝型号级支持。
- **协议证据 / 源码：** 固定来源 [`src/mchose/v3.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/v3.ts)、[`src/drivers/mchose/v3-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/v3-hid.ts)、[`docs/mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md)。文档记录真实 A7 V3 Ultra+ 8K 接收器上的 `0x0900` 身份/电量、`0x0002` 设置、`0x0003` DPI 和 `0x0001` 按键读取；**上游说明写入尚未实机验证**。本阶段只实现 `0x0900` 只读电池与连接状态，不读宏、不发 `0x01xx` 或其他写命令。由于固定上游文档给出该精确接收器路径的真实读取记录，允许无本机真机时发布明确标注未验证的只读 preview；若运行时产品字符串缺失或冲突则拒绝型号级声明。
- **接口产物：** `mchose-a7-v3-ultra-plus` 单独设备包及 `read-state` 适配器；返回电量、充电/连接状态、固件未知时留空。manifest 只列接收器连接和只读能力。
- **验收：** PID、usage、报告长、报告 ID、命令与校验严格匹配；`0x0900` 内 vendor id 为 MCHOSE、产品字符串精确匹配时才报告型号；失联/设备休眠、错误产品字符串、无校验回复、未知产品号都应安全失败；对真实目标鼠标完成多轮电量及连接状态对照后才升验证等级。
- **阻碍 / 退出条件：** 主机只暴露共享接收器名且无法用鼠标字符串或其他可信运行时信息验证精确型号时，只能交付不安装的探测记录。写入、改键、宏、底座 RGB 均不属于本项。

### 2. VGN Dragonfly F2 Master+ 未验证只读预览

- **状态 / 优先级：** 已发布 0.4.4-preview.2 只读插件；固定上游驱动精确声明 F2 Master+ 的接收器/有线 PID 和报告形状，可作为未实机验证只读 preview 的来源依据；preview 范围严格限于已公开读取。
- **目标 / 身份：** VGN Dragonfly F2 Master+；无线接收器 `3554:FB56`，有线 `3554:FB57`。需 `0xff02` 集合、report ID `0x08`。OpenMouse WebHID 读写 16 字节 payload，Windows HID 报告缓冲区通常为 17 字节（1 字节 report ID + 16 字节 payload）；移植时分别核对报告描述符和 API 缓冲区语义。F1 Pro (`F502/F503`) 和 F2 Pro Max (`FB3D/FB3E`) 不是此候选，也不因 VID 或所谓共用方案被纳入。
- **协议证据 / 源码：** 固定来源 [`src/vgn/index.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/vgn/index.ts)、[`src/drivers/vgn/hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/hid.ts)、[`src/drivers/vgn/protocol.test.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/vgn/protocol.test.ts)。代码含电量命令 `0x04`、校验和、16 字节传输与读配置实现；该驱动还含写设置和借用 Teevolution 的按键映射，均不纳入本只读项。上游单元测试的伪设备/捕获帧不证明某个本机固件已验证。
- **接口产物：** 固定上游驱动明确将这对 VID/PID、报告形状绑定到 Dragonfly F2 Master+，可据此先发布标注未实机验证的 `vgn-f2-master-plus` 只读 preview DLL/catalog 项，范围限电量、充电状态和固件版本；若运行时身份信息与该声明冲突，则拒绝连接。
- **验收：** 匹配 VID/PID、usage、report ID/长度；帧校验和、命令回显、异常/超时测试通过；不得仅靠 3554 或产品 PID 宣告 F2 Master+。固定来源与报告/校验验证通过，preview 对配置存储无写操作；有真机后再对照电量和充电状态并更新验证等级。
- **阻碍 / 退出条件：** 若 USB 身份、report 形状或公开实现无法把该 transport 可靠绑定到 F2 Master+，才停止出包；没有本机真机不阻止带未验证标记的只读 preview。后续 F1/F2 家族分别建候选，不共享身份表。

### 3. ATK F1 Ultimate 身份冲突排查后再做只读 preview

- **状态 / 优先级：** P3；协议通道有公开实现，但现有资料对这个名字对应的 USB ID 存在冲突，先解冲突再碰实际设备。
- **目标 / 身份：** 上游固定目录中 `ATK F1 Ultimate` 对应 `373B:102E`、CID/MID `2,42`；目录 usagePage 65284（`0xFF04`）与实际通用驱动要求的 `0xFF02:0002` 还存在差异；同目录不是实测保证。现有项目品牌页/研究记录另列过 `373B:1031`，在实物/原始枚举证据解决差异前，两个 ID 都不允许新包认领。需 report ID `0x08`。OpenMouse WebHID 使用 16 字节 payload，Windows HID 报告缓冲区通常为 17 字节（含 report ID）；运行时命令 `0x10` 的 CID/MID 响应 `2,42` 验明身份，之后才读取电量命令 `0x04`。有线与接收器模式分别核对，不因一个模式通过推广到另一模式。
- **协议证据 / 源码：** 固定来源 [`src/drivers/atk/products.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/atk/products.ts)、[`src/drivers/atk/device-catalog.generated.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/atk/device-catalog.generated.ts)、[`src/drivers/atk/hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/atk/hid.ts)。源码实现了 CID/MID 解歧、版本/电量/EEPROM 读取与大量设置写入；只复核 `0x10` 和 `0x04` 读取，不调用任何写操作。不能误套 `373B:1216` 的 `FF05` 协议，也不能按同品牌其他 ATK 的 CID/MID 认领。
- **接口产物：** 先交一份枚举/应答证据记录；只有 `373B:102E` 与现有记录中的 `373B:1031` 冲突经权威型号资料或实机枚举解决后，才单独生成 `atk-f1-ultimate` 电量只读 preview 插件，manifest 精确限定确认后的 transport。若冲突无法解决则停止该型号包。
- **验收：** 记录两台设备并排枚举差异、每个 HID 集合的 report ID/方向/长度、CID/MID 原始回复；确认 102E/1031 差异属于哪一具体产品/接收器；重复读取电量且无写帧。包装器对无法确认的 CID/MID 或另一 PID 返回“不支持”。
- **阻碍 / 退出条件：** PID 冲突未由硬件和可追溯资料解决前不出包；CID/MID 不是 `2,42`、报文不是预期长度/集合或无法区分有线/接收器时停止，不扩大到 F1 系列。

### 4. 对上游已有精确设备写入证据的设置能力做移植

- **状态 / 优先级：** P4；先从精确设备/连接/固件已有上游真实写入与恢复记录的单项能力中选一项移植。A7 V3 当前固定资料明确“写入未在硬件发送”，所以其设置写入暂不列入可发布候选；本条不要求本项目先拥有设备。
- **型号 / 传输 / 身份：** 以该操作的上游证据实际覆盖的精确型号、连接和固件范围为准；不把读能力推成写能力，不跨产品代际或相似 PID。
- **协议证据 / 源码：** 复用上游实现、抓包、读回和恢复结果，并在包中保留来源/许可证记录。若上游对该精确操作没有硬件证据，仍可做适配与离线验证，但不能把未证明的功能当作已验证能力发布。
- **接口产物：** 先接入一个证据充分的可回读设置动作；保留整块配置中的未知字段及其他设置。迁移范围优先使用现有 ABI/面板能力，不借机扩展无关协议。
- **验收：** 针对移植变化检查 Windows report ID 与缓冲区对应、命令/回复边界、ABI/manifest 能力声明、异常错误映射，并做安装/升级前后用户设置、配置和设备数据保留检查。上游已验证的设备交互不在本项目重演；只有移植边界或上游未覆盖的固件/连接差异需要新增硬件检查。
- **阻碍 / 退出条件：** 精确能力证据不足时只冻结该能力的发布声明，不阻止其他已有证据的型号/功能继续接入。用户数据在安装/升级中有丢失或覆盖风险时先修移植/安装路径；该项与协议是否成立分开处理。

### 5. 宏数据：按精确设备的上游读写/恢复证据决定接入范围

- **状态 / 优先级：** P5；当前固定资料没有证明现有 A5 Ultra (`3837:100C`) 宏格式可从 OpenMouse A5 Gen 1 (`2023` CompX) 协议复用，故不据此开放宏写入；这不是要求先在本项目硬件上重做上游协议验证。
- **候选型号：** 现有 `mchose-a5-ultra` 包所支持的初代 A5 精确设备及其当前验证 transport；不转移到 A5 V2、A7 或 A5 V3。现有品牌页记录：宏页不能完整备份，因此当前拒写。
- **来源 / 复用边界：** OpenMouse 固定源码的 `src/drivers/mchose/a5-gen1-hid.ts` 是另一套 `0x2023` CompX A5 Pro Max 协议，不能证明现有 `3837:100C` 初代 A5 Ultra 的宏格式。A7 V3 的 `readMacro` 资料也不能跨型号复用。
- **接口产物：** 后续若出现精确 A5 Ultra 的可追溯上游完整读写/恢复证据，按证据决定是只读导出还是编辑；上游没有该证据时保持未开放，不以别代协议填补。
- **验收：** 本项目检查移植是否保留宏/配置数据，并验证备份导入导出代码和异常路径；上游已对目标固件验证的宏事务无需重复。安装/升级数据保留必须有软件级前后对比。
- **阻碍 / 退出条件：** 当前证据无法确认该型号宏格式时，冻结这个型号的宏能力；其它设备上有证据的宏能力可独立评估。

### 6. 照明：独立的 MCHOSE MagDock 插件

- **状态 / 优先级：** 下一批候选优先项；与鼠标状态分开建项，灯在底座而非 A7 V2 鼠标本体。上游已有真实底座抓包、灯效读回、改值并恢复原状态记录，可按这项证据移植；无需等待本项目取得同款硬件。
- **目标 / 身份：** MCHOSE MagDock 充电底座，`3837:1012`、usage `0xFF00`；先用 HID 产品名与集合确认是底座。不得把效果发送给同 VID 的鼠标配置集合。
- **协议证据 / 源码：** 固定来源 [`src/mchose/dock.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/dock.ts)、[`src/drivers/mchose/dock-hid.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/drivers/mchose/dock-hid.ts)、[`docs/mchose-protocol.md` MagDock 部分](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md)。协议记录有真实底座效果抓包、读回和恢复结果，可作为移植证据；检查复用文件的 AGPL 来源与义务，不重复进行上游已有的底座硬件验证。
- **接口产物：** 独立 `mchose-mag-dock` 插件；若宿主元数据/ABI 尚不能给非鼠标设备提供 `lighting-read` / `lighting-apply` 能力，先提交能力扩展设计，不能伪装为鼠标 `read-state`。
- **验收：** 按上游定义移植全块读-改-写数据语义并保留未编辑字段；检查 Windows 报告映射、宿主 ABI 能力、安装/升级数据保留和界面设备类型。上游已做的底座实测/恢复不重复作为本项目准入测试；本项目包仍注明未在本机验证。
- **阻碍 / 退出条件：** 若当前宿主 ABI 不能表达底座的 `lighting-read` / `lighting-apply`，先记录所需最小能力接口；不得伪装为鼠标能力。不因本机暂时无底座而删除该候选；不影响既有 G2 Pro 灯效包。

## 本轮选择结论

快速接入仍按精确型号/连接/固件范围和所需能力分别判断。只读预览继续保持身份与协议范围限制；ATK F1 Ultimate 的 `102E` 与 `1031` 冲突未解前不认领。对有上游真实设备写入、读回和恢复记录的能力，可直接进入本项目移植；仅检查 Windows/HID、ABI、包集成与安装/升级用户数据保留，并在包中如实标注“上游硬件验证、本项目未实机验证”。本机缺少设备不构成禁入条件；上游未验证的操作继续列为研究项，不冒充验证能力。

## 下一批可复用功能候选

- **MCHOSE MagDock 底座灯效（优先）**：固定上游资料明确 `3837:1012`、`FF00`、无编号输出报告，记录真实底座灯效读取、整块写入、亮度/颜色修改、读回与恢复。可以复用现成协议/状态模型，移植时检查整块状态保留、Windows HID 映射及 ABI 是否能表达独立底座灯效；不要求本项目复测底座协议。来源见固定提交的 [`mchose-protocol.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/mchose-protocol.md) MagDock 章节及 [`src/mchose/dock.ts`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/src/mchose/dock.ts)。
- **AJAZZ AJ179 PRO 设置与侧键映射**：固定资料覆盖接收器 `3151:402D`、USB `3151:4026`、蓝牙 `3151:402C`，鼠标固件 v3.03，并记录三条路径的设置/按键读写、回读和配置恢复；可作为完整的多连接移植候选，按上游记录限定各传输能力。移植验收重点是连接路径映射、蓝牙事件报告边界、现有数据在安装/升级中的保留，不重跑上游设置协议。来源见固定提交 [`ajazz-aj179-pro.md`](https://github.com/OpenMouse-Project/mouse-protocol/blob/bac031567b28f2a55c616fc525b03f2d5fa452cc/docs/ajazz-aj179-pro.md)。

ATK A9 Plus Nearlink 当前可追溯到有线/接收器、鼠标 1.20/1.30 与接收器 1.20/1.23 的身份、电池读取抓包；这些证据支持既有只读范围，未单独证明其设置写入，不把写能力列为当前候选。A7 V3 写入同样因固定资料明确未在硬件发送而暂缓声明。




## 0.4.5 本轮增补

- 新增 ATK X1 Pro Max 只读预览，限有线 `373B:1017` 与 8K 接收器 `373B:101B`，读取 CID/MID `2,39` 后才读取电量。来源为固定 OpenMouse 的真实读取记录，避免从目录孤立数字推断协议集合；本项目仍未实机验证。详见 [X1 Pro Max](ATK-X1-PRO-MAX.md)。
- 新增 Razer Viper V2 Pro 有线 `1532:00A5` 只读预览；不推广至 `00A6` 或其他型号。详见 [Viper V2 Pro](RAZER-VIPER-V2-PRO.md)。
- F1 Ultimate 既有身份与接口疑问继续保留，不因为新增同品牌设备就解除限制。后续可优先调查 OpenMouse 已记录的 ATK A9 Plus Nearlink，但仍需按两代官方硬件与不同接口分别确认。

## 0.4.6 并行增补

新增 ATK A9 Plus Nearlink 有线及 NearLink 接收器只读预览，按 CID/MID `2,83` 及独立抓包核对；新增 Viper Ultimate `1532:007A` 有线只读预览。这两款同样未在本项目实机验证，仅开放电量等读取，不能从同品牌已有支持推导设置可写。主程序继续使用 0.4.5，无需为了安装这些插件再次升级主程序。
