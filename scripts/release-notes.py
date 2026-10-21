#!/usr/bin/env python3
"""Write short user-facing release notes from only the packaged catalog entries."""
import argparse
import json
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
            if entry["core_min"] != "0.4.0" or entry["core_max_exclusive"] != "0.5.0":
                raise ValueError("Update the release guidance before changing the core range")
            ids.add(entry["id"])
            entries.append(entry)
    if not entries:
        raise ValueError("Cannot publish release notes without a packaged device")

    lines = ["本次设备驱动更新：", ""]
    for entry in entries:
        name, version, changelog = (plain(entry[field]) for field in ("name", "version", "changelog"))
        preview = "（预览版）" if entry["channel"] == "preview" else ""
        lines.append(f"- {name} {version}{preview}：{changelog}")
        if entry["channel"] == "preview":
            if set(entry.get("capabilities", [])) == {"battery"}:
                lines.append("  当前仅查看电量，尚未实机验证。")
            elif entry.get("device", {}).get("validation", {}).get("status") == "public-implementation-untested":
                lines.append("  尚未实机验证，支持范围以设备说明为准。")
    lines.extend(["", "适用于 Windows x64 和 Peripheral Desk 0.4.x。在主程序的驱动商店按需安装需要的设备包即可。"])
    if any(entry["channel"] == "preview" for entry in entries):
        lines.append("安装预览设备前，请先在驱动商店切换到预览通道。")
    if any(entry["id"] == "hecate-g2-pro" for entry in entries):
        lines.append("G2 Pro 音效仍需本机已安装官方 HECATE / C-Media 组件，下载包不附带官方驱动。")
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
