# Protocol-fact references

The device plugins in this repository are independent implementations. The
projects below were used as references for protocol facts only (frame layouts,
report IDs, byte offsets, USB IDs); no source code was copied from them unless
a separate license file in this directory states otherwise.

- HeadsetControl (Sapd/HeadsetControl), GPL-3.0 — Corsair, HyperX Cloud II
  Wireless (both HP and Kingston editions, Void rich family), Arctis/Nova
  station layouts, Arctis 1, Arctis Nova 3P Wireless, Astro A50 Gen 4/Gen 5,
  Logitech G930/G933, Jabra GNP devices, Lenovo wireless VoIP, Plantronics
  BT600, Sony INZONE H5 and MCHOSE X9. Facts only.
- LennardKittner/HyperHeadset, MIT — HyperX Cloud III / Cloud III S framing.
  Facts only.
- yurtemre7/steel-mouse, MIT — SteelSeries Aerox/Rival reply decoding.
  Facts only.
- flozz/rivalcfg, WTFPL (see rivalcfg-WTFPL.txt) — Aerox 5/Rival 3 wireless
  battery exchange. Facts only.
- plugato/JBL_Baterry_Monitor (no license file published) — JBL Quantum 910
  report shapes. Facts only.
- andrewrabert/python-pulsar-mouse-tool, MIT — 17-byte Pulsar/ATK/VXE frames.
  Facts only.
- sopparus/hitscan-battery, MIT — Hitscan Hyperlight mapping. Facts only.
- z750sasr/mchose-a5-pro-max-web-driver — MCHOSE A5 Pro Max framing.
  Facts only.
- Klegus/mchose-macos — independent corroboration of the MCHOSE vendor id
  transport. Facts only.
- openmouse/protocol (OpenMouse) — VXE R1 Pro Max receiver usage page.
  Facts only.
- csutcliff/keychron-battery-dkms, GPL — Keychron Ultra-Link 8K / M5 status
  request. Facts only.
- Lofree web driver (hyzen.lofree.tech) — Lofree Hyzen transaction framing.
  Facts only.
- pwr-Solaar/Solaar, GPL — Logitech HID++ battery features and legacy
  registers. Facts only.
- openrazer/openrazer, GPL — Razer keyboard battery command parameters.
  Facts only.
- libsdl-org/SDL, Zlib — 8BitDo Pro 3, DualShock 4 and Steam Controller 2025
  puck input-report layouts. Facts only.
- HeyOkay/HaloBattery, MIT (see HaloBattery-MIT.txt) — protocol notes and
  hardware verification reports referenced per device in the catalog
  metadata. Facts only.
- seerge/g-helper, GPL-3.0 — ASUS ROG Harpe Ace Aim Lab Edition, Spatha X,
  Pugio II, Strix Impact II Wireless and Gladius III Wireless battery frame
  layouts. Facts only.
- [Vigil Watt](https://github.com/evren-oct/vigil-watt/tree/6315988f54e47fc030ae4c535dc64ae27c08f471), MIT, supplies the 0x4D receiver capture, device catalog, and configuration rate tables. The 16 MCHOSE receiver packages include the upstream MIT notice. See the [protocol evidence](../crates/protocol-mchose-0x4d/UPSTREAM.md).
- MagDock flow direction and the editable colors in Shining and Music are independent Rust implementations of protocol facts from the [manufacturer's M HUB frontend build](https://www.mchose.com.cn/assets/main-DIt5vAcV.js). The [device notes](../docs/MCHOSE-MAGDOCK.md) document the exact command fields; no vendor JavaScript is packaged with the plugin.

## Onboard and per-axis settings

OpenSynapse v1.7.2 (© 2026 A1mAssist, AGPL-3.0-only) documents the DeathAdder PID variants and onboard pointer reports. Logitech OMM serves as a feature reference. Solaar at e7304c4 and libratbag at 8235b5b document the HID++ protocol and exact model records. This implementation uses protocol and device facts. The linked model notes identify the original sources and licenses. [OpenSynapse](../docs/RAZER-OPENSYNAPSE.md) · [Logitech OMM and the HID++ implementations](../docs/LOGITECH-ONBOARD.md)
- MCHOSE M HUB's public frontend, official mouse front views and manuals supplied protocol and physical-layout facts for the 0x4D mouse binding diagrams, the K5 V2 receiver and the G75 Pro macro pages. The Rust implementations and diagrams were independently authored for this project. [Mouse sources](../crates/protocol-mchose-0x4d/UPSTREAM.md) · [G75 Pro source and verification](../drivers/mchose-g75-pro/UPSTREAM.md)

## MCHOSE 原生磁轴与音频设置

型号、接口、原版布局及参数依据迈从 M HUB 固定前端 `main-DIt5vAcV.js` 及磁轴编辑器构建 `bdf4d2c3`。布局和模板格式保留原作者迈从（MCHOSE）的来源。Rust 协议和 Win32 编辑界面由本项目实现。厂商资源按项目既有的开源、非商业使用条件处理。[音频实现范围](../crates/protocol-mchose-audio/UPSTREAM.md) · [型号名单](../docs/MCHOSE-NATIVE-FAMILIES.md)
