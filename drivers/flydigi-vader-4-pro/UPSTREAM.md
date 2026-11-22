# 飞智黑武士 4 Pro

将背部拨到 PC 档，长按 **O（FN）＋A** 切到蓝灯 DInput。在驱动管理搜索「黑武士 4 Pro」并安装，随后在「电量」页查看。长按 **O（FN）＋X** 可切回 XInput。电量按手柄档位换算，每档 20 个百分点。[官方切换说明](https://help.flydigi.com/docs/fk6fgtexhroeq72d)

| 连接 | 识别 | 验证范围 |
| --- | --- | --- |
| PC 档 DInput，2.4 GHz 或 USB | `04B4:2412` / `04B4:2414`、接口 02、32 字节 HID 输入、型号码 85／91／105 | InputPlumber 描述与开发者实机报告覆盖接口识别。omarchy-flydigi 有 2.4 GHz 协议样本。本项目的 Windows 实机读数待核对。 |
| PC 档 XInput | `045E:028E`，名称可能显示 Flydigi VADER4 | Windows 通用读数可能将低电手柄报为满电。插件使用 DInput。 |
| 蓝牙 | — | 此插件未适配蓝牙。 |

## 协议来源

- SDL 的 zlib 许可实现记录 `05 EC` 查询、32 字节回复、型号区分、电量档位、充电状态及有线标志。[SDL c8da179](https://github.com/libsdl-org/SDL/blob/c8da179691f909734cd1b124c930e49311428996/src/joystick/hidapi/SDL_hidapi_flydigi.c)
- InputPlumber 的设备描述记录 `04B4:2412` 接口 02。其开发者也报告过同型号的 `04B4:2414` 接口 02。[InputPlumber v0.83.0](https://github.com/ShadowBlip/InputPlumber/blob/v0.83.0/rootfs/usr/share/inputplumber/devices/60-flydigi_vader_4_pro.yaml) · [Valve 实机报告 #12103](https://github.com/ValveSoftware/steam-for-linux/issues/12103)
- omarchy-flydigi 记录经典 DInput 回复布局及黑武士 4 Pro 的 2.4 GHz 样本。[omarchy-flydigi dd4f4f6](https://github.com/z23/omarchy-flydigi/tree/dd4f4f67a2d9e90bad748d86195677d4129eea7d/helper/internal/vader)
- flydigi-space-station-mac 记录了接收器刚连接时可能暂报零电量的情况。插件遇到该状态显示电量未知。[flydigi-space-station-mac 649b9f3](https://github.com/uiltonlopes/flydigi-space-station-mac/tree/649b9f36cbba80deb7e7ac11309615d44f382a8e)
- HaloBattery 用户报告了同型号 XInput 低电误报满电。[HaloBattery #191](https://github.com/HeyOkay/HaloBattery/issues/191)

以上项目用于核对协议事实。Windows HID 实现使用外设盒已有的传输层。
