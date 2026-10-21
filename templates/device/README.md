# 新设备模板

将 Cargo.toml.template、lib.rs.template、adapter.rs.template 分别复制到 drivers/<新id>/Cargo.toml、src/lib.rs、src/adapter.rs。替换所有 example-device，加入 workspace，并将 metadata.json 条目加入根 drivers.json；添加 assets/<新id>.png。

metadata 中展示的是完整可选字段，不是已确认协议。只保留设备实际支持的 dpi、rates、sleeps、advanced；键盘或耳机不应带鼠标 DPI。Tuning key 必须使用宿主已认识的枚举名。若需新的设置种类，应先约定兼容模型，不能随意添加无法识别的枚举。

本模板使用自定义 adapter handler，新设备协议可在此仓库独立实现。不要简单把未知 id 转发给只认识现有四款设备的默认核心入口。新设备元数据和驱动 DLL 随包一起发布，宿主根据 metadata 创建页面。
