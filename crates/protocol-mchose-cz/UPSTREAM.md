# MCHOSE CZ wired magnetic keyboards

外设盒从原厂 CZ 编辑器构建 `bdf4d2c350a0fa0e93e443aa5375e26bc83cf419` 整理 USB 接口、板载设置字段、型号及实体键坐标。具体键盘和固件决定可选的轴体、行程范围、回报率、RT 精度、LT 和其他性能选项。[原厂 CZ 编辑器](https://www.mchose.com.cn/cizhou/index.html)

## 来源

| 原厂文件 | SHA-256 | 内容 |
| --- | --- | --- |
| [1266-83f12db42198b859.js](https://www.mchose.com.cn/cizhou/_next/static/chunks/1266-83f12db42198b859.js) | `b9529a6a85903b0517fba447df493288ab16110086a2801ef9116b6e07de15a5` | HID 数据包、板载区域、触发行程与高级键编码 |
| [3050-4fde3581e71b2bbc.js](https://www.mchose.com.cn/cizhou/_next/static/chunks/3050-4fde3581e71b2bbc.js) | `bc45754f208d93ab73d6ebf42471433ba9f9e61be26e9b6e88cb6b11c5d3df0d` | 设备 ID、型号与固件条件、轴体与键码 |
| [3996-ce02a54ba1430ebd.js](https://www.mchose.com.cn/cizhou/_next/static/chunks/3996-ce02a54ba1430ebd.js) | `dc05bcafc0f2770390005b4d649204276ac93ce57354a770654befe7ff7f44f4` | 官方键盘模板 v3 和配置应用 |
| [9849-a754f28a0e72206e.js](https://www.mchose.com.cn/cizhou/_next/static/chunks/9849-a754f28a0e72206e.js) | `d937a98dedc3ef8cb04c6fe5fe9c151993e9718bd34925dfbbf5c62aefa02638` | 高级键设置流程与布局 |


板载协议在本仓库用 Rust 实现。设备名称、轴体的显示名称和实体键坐标来自 MCHOSE。原站文本、商标和图形权利归原权利人，项目收到的使用说明为开源、非商业使用。各型号的插件只携带自己的键位表；原厂编辑器脚本不随这些插件分发。

## 接入范围

使用有线 USB、Usage Page 0001 / Usage 0000、无编号输入输出报告，Windows 缓冲区各 65 字节。板载每套配置有四层键位。导入模板会先读当前槽，匹配模板的具体型号后写入当前槽，遇到未知动作或共享槽冲突时提示保留。修改后的每段设置逐段读回；写入出错时尝试恢复原值并报告结果。

下表的上游依据是具体型号的原厂实现。本项目验证了数据包、设置与模板转换、回读和异常恢复；尚未取得这些型号的实机写入报告。部分布局和默认矩阵通过原站共用模块引用。原站 32 份布局文件包含 6 个机械键盘型号，本批共 26 个磁轴型号、1874 个可点选键。USB 引导程序和其他连接方式未列入。

| 型号插件 | 设备类型 | VID:PID | 实体键 |
| --- | ---: | --- | ---: |
| mchose-ace-60 | 101 | `41E4:2101` | 61 |
| mchose-ace-60-pro | 102 | `41E4:2103` | 61 |
| mchose-ace-60-pro-iso-fr | 150 | `3837:3040` | 62 |
| mchose-ace-60-pro-nordic | 109 | `3837:3002` | 62 |
| mchose-ace-60x-i | 103 | `41E4:2126` | 61 |
| mchose-ace-60x-ii | 104 | `41E4:2112` | 61 |
| mchose-ace-68-air-2 | 214 | `3837:300A` | 72 |
| mchose-ace-68-air-ii | 106 | `41E4:2120` | 68 |
| mchose-ace-68-air-iii | 202 | `41E4:2132` | 68 |
| mchose-ace-68-gt | 213 | `3837:3007` | 69 |
| mchose-ace-68-i | 111 | `41E4:2114` | 68 |
| mchose-ace-68-iii | 203 | `3837:3003` | 68 |
| mchose-ace-68-pro | 112 | `41E4:2116` | 68 |
| mchose-ace-68-turbo-16k | 211 | `3837:3026` | 69 |
| mchose-ace-68-turbo-8k | 212 | `3837:3028` | 70 |
| mchose-ace-68-v2-iii | 210 | `3837:3024` | 71 |
| mchose-ace-75-16k | 135 | `3837:301D` | 86 |
| mchose-ace-75-8k | 134 | `3837:303C` | 86 |
| mchose-god-60 | 136 | `3837:3020` | 66 |
| mchose-gt-75 | 138 | `3837:302D` | 83 |
| mchose-jet-75-i | 121 | `41E4:2118` | 80 |
| mchose-jet-75-ii | 122 | `41E4:211A` | 80 |
| mchose-jet-75-iii | 200 | `3837:3008` | 80 |
| mchose-mix-87-i | 108 | `41E4:2122` | 87 |
| mchose-mix-87-iii | 201 | `3837:300D` | 87 |
| mchose-zero-75x | 123 | `41E4:211C` | 80 |

支持的板载项目包括逐键行程、RT 触发与释放、上下死区、轴体、回报率、极限模式、键盘性能和 DKS、MT、TGL、CB、RS、SOCD。LT 按固件开放。高级键可选 Windows、Windows Fn、Mac、Mac Fn 层。导入 M HUB KeyboardProfile v3 的受支持范围为逐键触发、性能、键位、关联的高级键槽；模板中的宏和灯效由各自页面处理，未知数据在导入文件中保留。

原厂的手动校准依赖持续的实时按键行程报告与结束校准操作。当前原生页提供自动校准开关，手动引导校准和实时行程曲线仍由本地 M HUB 提供。
