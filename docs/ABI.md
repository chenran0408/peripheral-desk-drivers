# Driver ABI 1

DLL 为普通 Windows x64 用户态插件，不是内核驱动。宿主先校验下载包和 DLL，再加载；每款设备必须在其 manifest 中声明 ABI 1。

```c
#include <stdint.h>
#include <stddef.h>
typedef struct { uint8_t *ptr; size_t len; } PdBuffer;
uint32_t pd_driver_abi(void); /* always 1 */
PdBuffer pd_driver_request(const uint8_t *utf8_json, size_t len);
void pd_driver_free(uint8_t *ptr, size_t len);
```

输入只在调用期间借用，不带末尾 NUL。宿主负责提供可读内存，长度至多 1 MiB。返回是 UTF-8 JSON 的 Box<[u8]>，宿主读取后必须使用**同一个 DLL**的 pd_driver_free(ptr,len) 释放，且只释放一次。禁止跨 DLL 释放，禁止更改长度，禁止卸载仍有活动请求或未释放 Buffer 的 DLL。宿主与 DLL 指针宽度均为 x64。

FFI 层检查空指针和 UTF-8，并用 catch_unwind 捕获 Rust panic；有效内存指针仍是宿主的责任。异常返回 `{"ok":false,"value":null,"error":"..."}`。构建必须保留 `panic="unwind"`；无法捕获 abort、进程终止或无效内存访问。ABI 调用是同步的，宿主应在后台串行执行同一设备请求。

## 请求

从驱动包0.4.1起，现有四个 wrapper 将 JSON 交给固定 SDK 中各自的专用协议入口，只链接本设备所需功能；旧的全设备 dispatcher 不再是新包的必经入口。响应仍统一为：

```json
{"ok":true,"value":null,"error":""}
```

| op | 请求附加字段 | value |
| --- | --- | --- |
| `read-state` | 无 | State |
| `apply-setting` | `action`：核心 Action 的 serde 表示 | null |
| `controls-read` | 无 | ControlState |
| `controls-apply` | `action`：ControlAction | ControlState |
| `fx-query` | G2 Pro：可选 action 或 values，二者不可同时提供 | 音效 Reply |
| `lighting-read` | G2 Pro：无 | LightingState |
| `lighting-apply` | G2 Pro：config | LightingState |
| `lighting-switch` | G2 Pro 0.4.2 起：enabled（bool），仅切换硬件开关，不发送模式和色表 | LightingState |
| `fx-worker` | G2 Pro 兼容旧 CLI 的内部入口：request | 音效 Reply |

示例只读请求：`{"op":"read-state"}`。未知操作必须返回 ok=false，不能静默成功。State、Action、Tuning、ControlState 的具体 serde 定义以 Cargo.toml 固定 rev 的核心源码为准。每包 wrapper 固定自己的 device_key，输入不能改成其他设备。

SDK 调用在宿主启动的单层 plugin-worker 中执行，由宿主控制超时与结束进程；DLL 不再自行启动另一层音效进程。CI 的 ABI 探测只发送无效 UTF-8，在 FFI 校验处返回，不能把它当作设备协议或硬件功能测试。

新增设备可使用 `export_driver!("example-device", crate::adapter::request)`，自定义 Rust handler 签名为 `fn(&str,&str)->String`。它负责自己的设备匹配、协议和同样的 JSON schema。宿主从包 metadata 渲染设备能力，无需为了新品牌重写页面。包版本可独立增长；ABI仍为1、核心兼容范围仍为0.4.x，不因驱动版本从0.4.0升为0.4.1而改变。

新增可选操作必须同步声明 `capabilities`；宿主仅在能力存在时调用。G2 的即时开关界面需要主程序和插件均为 0.4.2；旧版应用仍可使用原有 `lighting-apply`。0.4.2 桌面注册表只由已安装的包元数据生成，不预装或默认显示开发机的四款设备。
