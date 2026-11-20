#!/usr/bin/env python3
"""Write short user-facing release notes from only the packaged catalog entries."""
import argparse
import json
import re
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
            if not re.fullmatch(r"0\.4\.(0|[1-9][0-9]*)", entry["core_min"]) or entry["core_max_exclusive"] != "0.5.0":
                raise ValueError("Update the release guidance before changing the core range")
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
    if any("dpi-stages-read" in entry["capabilities"] for entry in entries):
        lines.extend(["", "安装外设盒 0.4.11-preview.10 或更新版。打开设备详情即可读取 DPI 档位和回报率，也可进入官方 M HUB 修改设置。"])
    minimum = max((entry["core_min"] for entry in entries), key=lambda v: tuple(map(int,v.split("."))))
    lines.extend(["", f"适用于 Windows x64；请先将 Peripheral Desk 更新到 {minimum} 或更新的 0.4.x 版本，再在驱动商店按需安装。未列出的设备包不受影响。"])
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
