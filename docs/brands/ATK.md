# ATK

新增 **X1 Pro Max 0.4.5-preview.1 只读预览**：仅 `373B:1017` 有线和 `373B:101B` 8K 接收器，必须读到 CID/MID `2,39`。本项目尚未实机验证，不开放设置写入。详见 [X1 Pro Max 范围](../ATK-X1-PRO-MAX.md)。

暂缓候选 F1 Ultimate（以下为存在冲突的历史线索，不作支持声明）：公开表列 373B:1031 接收器、373B:102E 有线，FF02:0002、17字节 Input/Output。OpenMouse ATK 自身还按运行时 CID/MID 区分型号、传感器和能力；同一个 VID/PID 可能不足以识别。

需要进一步核对 exact 后缀/接收器、CID/MID、report08/command04 电量只读事务、应答校验与休眠行为。2.4G-battery-viewer 的373B:1216/FF05特性协议是另一组实机证据，不能套用到上述17字节通道；ATK Zero/Bitmouse等64字节协议也须另开适配器。没有公开完整型号身份时只能识别候选，不宣称电量已可用。

完整来源与许可证见 [调查矩阵](../PROTOCOL-RESEARCH.md)。
