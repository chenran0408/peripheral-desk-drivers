#!/usr/bin/env python3
"""Write short user-facing release notes from only the packaged catalog entries."""
import argparse
import json
from core_range import check_range, minimum_order
from pathlib import Path


def plain(value):
    if not isinstance(value, str) or not value.strip():
        raise ValueError("Release note fields must be nonempty strings")
    return " ".join(value.split())


def generate(directory):
    entries = []
    for channel in ("stable", "preview"):
        catalog = json.loads((directory / f"catalog-{channel}.json").read_text(encoding="utf-8"))
        if catalog.get("schema") != 1 or catalog.get("channel") != channel:
            raise ValueError("Release catalog schema/channel mismatch")
        ids = set()
        for entry in catalog["drivers"]:
            if entry["channel"] != channel or entry["id"] in ids:
                raise ValueError("Duplicate or misplaced release package")
            check_range(entry)
            ids.add(entry["id"])
            entries.append(entry)
    if not entries:
        raise ValueError("Cannot publish release notes without a packaged device")

    lines = ["本次更新", ""]
    groups = {}
    for entry in entries:
        group = (plain(entry["version"]), plain(entry["changelog"]))
        groups.setdefault(group, []).append(plain(entry["name"]))
    for (version, changes), names in groups.items():
        if len(names) == 1:
            lines.append(f"- **{names[0]}** {version}。{changes}")
        else:
            lines.append(f"- **{len(names)} 款型号**。{changes}")
            lines.extend(["", "  " + "、".join(names), ""])
    if any("onboard-profiles" in entry["capabilities"] for entry in entries):
        lines.extend(["", "罗技板载配置需要外设盒 0.4.11-preview.17 或更新版。在「设备调节」选择自己的鼠标。"])
    if any("dpi-stages-write" in entry["capabilities"] and entry["id"].startswith("razer-") for entry in entries):
        lines.extend(["", "雷蛇用户请进入「设备调节 → 对应型号 → 指针与性能」选择要修改的档位和横轴或纵轴。"])
    minimum = max((entry["core_min"] for entry in entries), key=minimum_order)
    lines.extend(["", f"适用于 Windows x64；请先将 Peripheral Desk 更新到 {minimum} 或更新的 0.4.x 版本，再在「插件管理」按需安装。未列出的设备包不受影响。"])
    if any(entry["channel"] == "preview" for entry in entries):
        lines.append("安装预览设备前，请先在驱动商店切换到预览通道。")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog_directory", type=Path, nargs="?", default=Path("dist"))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    text = generate(args.catalog_directory)
    output = args.output or args.catalog_directory / "release-notes.md"
    output.write_text(text, encoding="utf-8")
    print("Generated device release notes:", output)


if __name__ == "__main__":
    main()
