# OpenRazer battery adapters in wave 7

Source: OpenRazer commit `477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d`, [mouse device definitions](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/daemon/openrazer_daemon/hardware/mouse.py) and [battery command switch](https://github.com/openrazer/openrazer/blob/477f4d6d3ab7e9deb6310d5bd3742560c5cbe80d/driver/razermouse_driver.c#L1590-L1692).

| Device | Receiver PID | Battery transaction ID |
| --- | --- | --- |
| Atheris | 1532:0062 | 1F |
| Lancehead Wireless receiver | 1532:006F | 1F |
| Mamba Wireless receiver | 1532:0072 | 3F |
| Orochi V2 | 1532:0094 | 1F |
| DeathAdder V2 X HyperSpeed | 1532:009C | 1F |
| Pro Click | 1532:0077 | 1F |
| Basilisk Ultimate | 1532:0088 | 1F |
| Naga V2 Pro | 1532:00A8 | 1F |
| Viper V2 Pro | 1532:00A6 | 1F |
| DeathAdder V2 Pro | 1532:007D | 3F |
| Basilisk X HyperSpeed | 1532:0083 | FF |

The query is a 91-byte feature report, class 07 command 80; the raw battery byte is response argument 1 and scales from 0..255. These adapters use protocol facts, not copied GPL source. OpenRazer's driver is GPL-2.0-or-later. Upstream code confirms the listed commands and IDs, but no per-model physical battery capture was found; Peripheral Desk has not tested the hardware. Charging is deliberately not reported by these adapters.
