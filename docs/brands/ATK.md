# ATK

当前为调查候选，**本轮没有新 ATK 可安装支持声明**。

优先候选 F1 Ultimate：公开表列 373B:1031 接收器、373B:102E 有线，FF02:0002、17字节 Input/Output。OpenMouse ATK 自身还按运行时 CID/MID 区分型号、传感器和能力；同一个 VID/PID 可能不足以识别。

需要进一步核对 exact 后缀/接收器、CID/MID、report08/command04 电量只读事务、应答校验与休眠行为。2.4G-battery-viewer 的373B:1216/FF05特性协议是另一组实机证据，不能套用到上述17字节通道；ATK Zero/Bitmouse等64字节协议也须另开适配器。没有公开完整型号身份时只能识别候选，不宣称电量已可用。

完整来源与许可证见 [调查矩阵](../PROTOCOL-RESEARCH.md)。
