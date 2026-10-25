# 迈从 V9 Pro 耳机电量预览

本插件只读取电量，不控制灯效、EQ、麦克风或其他功能。

## 精确连接范围

- 仅 USB 2.4 GHz 接收器 `291D:385D`。
- 仅命中路径包含 `&mi_00&col05#`、usage `FF90:0001`、64 字节输入/输出、无 Feature 报告的电量 collection。
- 其他 collection、蓝牙、V9 其他版本、Pro2、Turbo 均不在支持范围。无法唯一匹配时不发送查询。

插件安装后会定期主动发出 `55 65 01` 查询帧；它不是被动监听。预览包需用户主动选择安装。只接受 64 字节、report ID `55`、命令 `65` 且电量字节 `[2]` 在 2–100 的响应。电量是原始整数百分比，不做单位换算或插值；完整查询帧回显、0、1、超范围值、短帧和其他命令都拒绝。`[3]` 的语义未确认，充电状态保持未知。

## 验证与限制

Vigil Watt v1.3.0（`a78e973b4eb634bdce37c9078dd48d1c07e3eb73`）记录一名贡献者在一台 V9 Pro 上抓到的官方 MCHOSE HUB 报文，样本电量为 100%。Vigil Watt 维护者未复测；这不是 Peripheral Desk 本机验证。非满电区间的映射也没有样本，所以本插件仅标为上游单点验证、本项目未实测。

本插件将 Vigil Watt v1.3.0 的 `MchoseV9ProCodec` 查询/解析规则和 `MchosePlugin.QueryHeadsetV9` 轮询窗口移植为 Rust，统一到 Peripheral Desk 插件 ABI。来源仓库为 MIT 许可；原版权归属和完整许可文本随源码及本插件 ZIP 一同提供。详情见 [Vigil Watt V9 Pro 协议记录](https://github.com/evren-oct/vigil-watt/blob/a78e973b4eb634bdce37c9078dd48d1c07e3eb73/docs/hardware/mchose-v9-pro.md) 与 [MIT 许可](https://github.com/evren-oct/vigil-watt/blob/a78e973b4eb634bdce37c9078dd48d1c07e3eb73/LICENSE)。
