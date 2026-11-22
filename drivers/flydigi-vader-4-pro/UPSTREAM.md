# 飞智黑武士 4 Pro

将背部拨到 PC 档，长按 **O（FN）＋A** 切到蓝灯 DInput。在驱动管理搜索「黑武士 4 Pro」并安装，随后在「电量」页查看。长按 **O（FN）＋X** 可切回 XInput。电量按手柄档位换算，每档 20 个百分点。[官方切换说明](https://help.flydigi.com/docs/fk6fgtexhroeq72d)

白灯 XInput 模式下，先安装飞智空间站，确认空间站能识别手柄，再退出空间站界面并留着后台服务。外设盒从同一个后台服务读取电量，游戏继续使用 XInput。[空间站官方下载](https://www.flydigi.com/index/down?nav_id=2)

| 连接 | 识别 | 验证范围 |
| --- | --- | --- |
| PC 档 DInput，2.4 GHz 或 USB | `04B4:2412` / `04B4:2414`、接口 02、32 字节 HID 输入、型号码 85／91／105 | InputPlumber 描述与开发者实机报告覆盖接口识别。外设盒在 Windows 本机通过 `2412` 接收器读到了 4 档（80%）。 |
| PC 档 XInput | `045E:028E`，USB 父设备必须为 `Flydigi_VADER4`，还需有空间站的型号与连接记录 | 已核对本机的 USB 身份。电量走空间站本地服务，实机读数待核对。 |
| 蓝牙 | — | 此插件未适配蓝牙。 |

## 协议来源

- SDL 的 zlib 许可实现记录 `05 EC` 查询、32 字节回复、型号区分、电量档位、充电状态及有线标志。[SDL c8da179](https://github.com/libsdl-org/SDL/blob/c8da179691f909734cd1b124c930e49311428996/src/joystick/hidapi/SDL_hidapi_flydigi.c)
- InputPlumber 的设备描述记录 `04B4:2412` 接口 02。其开发者也报告过同型号的 `04B4:2414` 接口 02。[InputPlumber v0.83.0](https://github.com/ShadowBlip/InputPlumber/blob/v0.83.0/rootfs/usr/share/inputplumber/devices/60-flydigi_vader_4_pro.yaml) · [Valve 实机报告 #12103](https://github.com/ValveSoftware/steam-for-linux/issues/12103)
- omarchy-flydigi 记录经典 DInput 回复布局及黑武士 4 Pro 的 2.4 GHz 样本。[omarchy-flydigi dd4f4f6](https://github.com/z23/omarchy-flydigi/tree/dd4f4f67a2d9e90bad748d86195677d4129eea7d/helper/internal/vader)
- flydigi-space-station-mac 记录了接收器刚连接时可能暂报零电量的情况。插件遇到该状态显示电量未知。[flydigi-space-station-mac 649b9f3](https://github.com/uiltonlopes/flydigi-space-station-mac/tree/649b9f36cbba80deb7e7ac11309615d44f382a8e)
- HaloBattery 用户报告了同型号 XInput 低电误报满电。[HaloBattery #191](https://github.com/HeyOkay/HaloBattery/issues/191)
- BatteryMonitor 和飞智手柄电量托盘公开了空间站本地管道的只读设备列表请求、返回字段与单客户端行为。[BatteryMonitor 25c2213](https://github.com/JKWTCN/BatteryMonitor/blob/25c221326391e26b62ae2b7db8e72ce8538ff909/scripts/flydigi_controller_battery.py) · [飞智手柄电量托盘 c1afd9b](https://github.com/wutong233/flydigi-battery/blob/c1afd9bf2de713d1d9d1e8f5c1f8192dc737cdab/docs/protocol.md)

以上项目用于核对协议事实。Windows HID 实现使用外设盒已有的传输层。
