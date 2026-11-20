#!/usr/bin/env python3
"""Generate the research index from the installable plugins, excluding unreleased leads."""
import argparse
import json
import re
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "catalog"
CHANNEL_FILES = {"stable": "drivers.json", "preview": "drivers-preview.json"}

STATUS = {
    "hardware-verified": "本项目实机",
    "upstream-hardware-verified-project-untested": "上游实机",
    "upstream-verified": "上游验证",
    "public-implementation-untested": "公开实现",
    "code-adapter-only": "代码适配",
}
CAPABILITIES = {
    "instance-target": "按实物定向", "keymap-home": "可视改键首页", "control-sections": "按页读取键位与灯光",
    "battery": "电量", "settings": "设置", "bindings": "改键", "resident-macros": "常驻宏",
    "lighting-switch": "灯光开关", "lighting-editor": "灯效编辑", "audio": "音效",
    "lighting-direction": "流动方向",
    "polling-rate-read": "读配置回报率", "dpi-stages-read": "读 DPI 档位配置",
    "polling-rate-write": "设置回报率", "dpi-stages-write": "编辑 DPI 档位",
    "pointer-controls": "指针设置", "onboard-profiles": "板载模式、配置与 DPI 档位切换",
    "experimental-onboard-macros": "板载宏（实验）", "board-name": "配置改名",
}
POLICIES = {"manual": "手动认领", "runtime-identity": "运行时核对", "interface": "接口匹配"}


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def json_text(value):
    return json.dumps(value, indent=2, ensure_ascii=False) + "\n"


def vendor_key(brand):
    return re.sub(r"[^\w]+", "-", brand.casefold()).strip("-")


def family_assignments(research):
    assignments = {}
    for vendor, record in research.items():
        family_ids = set()
        for family in record["families"]:
            if family["id"] in family_ids:
                raise ValueError(f"Duplicate protocol family {vendor}/{family['id']}")
            family_ids.add(family["id"])
            for model in family["models"]:
                if model in assignments:
                    raise ValueError(f"{model} appears in multiple curated families")
                assignments[model] = (vendor, {k: v for k, v in family.items() if k != "models"})
    return assignments


def implementation(plugin):
    crate = ROOT / "drivers" / plugin
    deps = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))["dependencies"]
    shared = sorted({
        (crate / dep["path"]).resolve().relative_to(ROOT).as_posix()
        for dep in deps.values() if isinstance(dep, dict)
        and Path(dep.get("path", "")).name.startswith("protocol-")
    })
    core = deps.get("peripheral-desk-driver-ffi", {}).get("features", [])
    if shared:
        return shared
    if core:
        return [f"crates/driver-ffi#{feature}" for feature in sorted(core)]
    return [f"drivers/{plugin}"]


def inferred_family(plugin, paths):
    if len(paths) == 1 and paths[0].startswith("crates/"):
        name = paths[0].rsplit("/", 1)[-1].split("#")[-1]
        return {"id": name, "name": name, "basis": "implementation"}
    return {"id": plugin, "name": plugin, "basis": "model"}


def build():
    research = load_json(CATALOG / "families.json")
    assignments = family_assignments(research)
    vendors = {}
    rows = {}
    brands = defaultdict(set)
    for channel, filename in CHANNEL_FILES.items():
        for package in load_json(ROOT / filename):
            plugin = package["id"]
            device = package["device"]
            key = vendor_key(device["brand"])
            brands[key].add(device["brand"])
            if (plugin, channel) in rows:
                raise ValueError(f"Duplicate {channel} plugin {plugin}")
            rows[plugin, channel] = package
            vendor = vendors.setdefault(key, {"id": key, "families": {}})
            paths = implementation(plugin)
            if plugin in assignments:
                claimed_vendor, family = assignments[plugin]
                if claimed_vendor != key:
                    raise ValueError(f"{plugin} is catalogued under the wrong vendor")
                family = {**family, "basis": "documented"}
            else:
                family = inferred_family(plugin, paths)
            entry = vendor["families"].setdefault(family["id"], {**family, "models": []})
            validation = device["validation"]
            connections = validation.get("connections", [])
            if isinstance(connections, str):
                connections = [connections]
            policy = "manual" if device.get("manual") else "runtime-identity" if device.get("runtime_identity") else "interface"
            model = {
                "id": plugin, "name": device["name"], "kind": device["kind"], "channel": channel,
                "capabilities": package["capabilities"], "identity": policy,
                "validation": validation["status"], "implementation": paths,
                "connections": connections, "firmware": validation.get("firmware", "unknown"),
            }
            if device.get("manual_hint"):
                model["manual_hint"] = device["manual_hint"]
            entry["models"].append(model)
    missing = set(assignments) - {plugin for plugin, _ in rows}
    if missing:
        raise ValueError(f"Unknown curated plugin IDs: {sorted(missing)}")
    for key, vendor in vendors.items():
        # Prefer the packaging's mixed-case spelling if both cases exist.
        vendor["name"] = sorted(brands[key], key=lambda name: (name.isupper(), name))[0]
        vendor["evidence"] = research.get(key, {}).get("evidence", [])
        families = vendor["families"]
        for family in families.values():
            family["models"].sort(key=lambda m: (m["id"], m["channel"]))
        vendor["families"] = [families[f] for f in sorted(families)]
    index = {"schema": 1, "manifests": CHANNEL_FILES, "vendors": [vendors[k] for k in sorted(vendors)]}
    return index, rows


def browse(index, rows):
    def ref(vendor):
        return "vendor-" + vendor["id"]

    lines = [
        "# 按厂商查找外设", "",
        f"共 {len(index['vendors'])} 个品牌、{len(rows)} 个插件条目。请在外设盒驱动商店选择对应通道，搜索型号并安装。",
        "", " · ".join(f"[{v['name']}](#{ref(v)})" for v in index["vendors"]), "",
        "型号、连接方式和功能由各包独立声明。点开型号可查看源文件；验证记录只覆盖该包描述的连接方式与功能。接口匹配仍可能命中多个候选，由驱动商店区分。其他厂商的协议族可来自共享实现，未经单独确认的型号保留独立条目。",
        "", "维护及完整型号查询见[贡献指南](../CONTRIBUTING.md#vendor-research)。结构化索引见 [vendors.json](vendors.json)。",
        "原厂登记型号和上游协议规格可在[厂商研究资料](upstream/vigil-watt/README.md)中查阅。",
    ]
    for vendor in index["vendors"]:
        lines.extend(["", f'<a id="{ref(vendor)}"></a>', "", f"## {vendor['name']}", ""])
        if vendor["id"] == "mchose":
            lines.extend(["[迈从各代接收器与厂商研究记录](../docs/brands/MCHOSE.md)", ""])
        lines.extend(["| 协议 / 实现 | 型号 | VID:PID | 功能 | 通道 | 识别方式 | 验证 |",
                      "| --- | --- | --- | --- | --- | --- | --- |"])
        for family in vendor["families"]:
            for model in family["models"]:
                plugin, channel = model["id"], model["channel"]
                device = rows[plugin, channel]["device"]
                ids = dict.fromkeys(f"{m['vid']:04X}:{m['pid']:04X}" for m in device["matches"])
                caps = "、".join(CAPABILITIES.get(c, c) for c in model["capabilities"])
                path = ROOT / "drivers" / plugin
                evidence = next((p for p in ("UPSTREAM.md", "PROVENANCE.md", "README.md") if (path / p).is_file()), "")
                url = "../drivers/" + plugin + ("/" + evidence if evidence else "")
                title = model['name'].replace("|", r"\|")
                rowspec = [family["name"], f"[{title}]({url})", " ".join(ids), caps,
                           "稳定" if channel == "stable" else "预览",
                           POLICIES[model["identity"]], STATUS.get(model["validation"], model["validation"])]
                lines.append("| " + " | ".join(rowspec) + " |")
    return "\n".join(lines) + "\n"


def generated_files(index, rows):
    return {CATALOG / "vendors.json": json_text(index), CATALOG / "README.md": browse(index, rows)}


def check_current():
    for path, content in generated_files(*build()).items():
        if not path.is_file() or path.read_text(encoding="utf-8") != content:
            raise ValueError(f"Outdated {path.name}. Run python scripts/vendor_catalog.py --write")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true")
    group.add_argument("--check", action="store_true")
    group.add_argument("--model", metavar="PLUGIN_ID", help="print model-specific interfaces, capabilities, scope and evidence")
    args = parser.parse_args()
    if args.check:
        check_current()
        print("Vendor catalog is up to date")
        return
    index, rows = build()
    if args.model:
        detail = []
        for vendor in index["vendors"]:
            for family in vendor["families"]:
                for model in family["models"]:
                    if model["id"] == args.model:
                        detail.append({"vendor": vendor["name"], "family": family["name"],
                                       "basis": family["basis"], "implementation": model["implementation"],
                                       "package": rows[model["id"], model["channel"]]})
        if not detail:
            parser.exit(1, f"No plugin with ID {args.model}\n")
        sys.stdout.buffer.write(json_text(detail).encode("utf-8"))
        return
    for path, content in generated_files(index, rows).items():
        path.write_text(content, encoding="utf-8", newline="\n")
    print(f"Indexed {len(rows)} plugin entries across {len(index['vendors'])} vendors")


if __name__ == "__main__":
    main()
