"""Refresh vendor research from Vigil Watt. Reports are independent of the install catalog."""
import argparse
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "catalog/upstream/vigil-watt"
REPO = "https://github.com/evren-oct/vigil-watt"
PLUGINS = "src/VigilWatt.Core/Plugins"
BRANDS = "docs/hardware/brands"
QUOTED = r'"(?:\\.|[^"\\])*"'
SECTIONS = ("devices", "officialDevices", "transports", "receivers", "protocols", "rateTables",
            "rateProfiles", "voltageCurves", "wpidCurves")


def parse(raw):
    """Parse upstream JSONC, leaving comment markers and commas inside strings intact."""
    def keep_string(match):
        return match[0] if match[0].startswith('"') else ""
    data = raw.decode("utf-8-sig")
    data = re.sub(QUOTED + r"|//[^\r\n]*|/\*.*?\*/", keep_string, data, flags=re.S)
    return json.loads(re.sub(QUOTED + r"|,(?=\s*[}\]])", keep_string, data))


def dump(obj):
    return (json.dumps(obj, indent=2, ensure_ascii=False) + "\n").encode()


def git(checkout, *args):
    return subprocess.check_output(["git", "-C", str(checkout), *args])


def resolve(checkout, ref):
    return git(checkout, "rev-parse", "--verify", ref + "^{commit}").decode().strip()


def git_tree(checkout, ref, base):
    entries = git(checkout, "ls-tree", "-r", "-z", ref, "--", base).decode().split("\0")
    return {name: head.split()[2] for row in entries if row for head, name in [row.split("\t", 1)]}


def usb(row):
    return f"{int(row.get('vid', '046d'), 16):04X}:{int(row['pid'], 16):04X}"


def entries(doc):
    rows = {}
    for section in SECTIONS:
        value = doc.get(section, {})
        if isinstance(value, dict):
            rows.update(((section, key), item) for key, item in value.items())
        else:
            for item in value:
                key = item["pid"].lower() if section == "officialDevices" else usb(item)
                token = (section, key)
                if token in rows:
                    raise ValueError(f"Duplicate upstream identity in {section}: {key}")
                rows[token] = item
    return rows


def differences(name, old, new):
    before, after = entries(parse(old)) if old else {}, entries(parse(new)) if new else {}
    if old == new:
        return []
    result = []
    for token in sorted(before.keys() | after.keys()):
        left, right = before.get(token), after.get(token)
        if left == right:
            continue
        symbol = "+" if left is None else "-" if right is None else "~"
        item = right if right is not None else left
        model = item.get("model", item.get("familyDisplay", "")) if isinstance(item, dict) else ""
        changes = ""
        if isinstance(left, dict) and isinstance(right, dict):
            changes = " (" + ", ".join(sorted(k for k in left.keys() | right.keys() if left.get(k) != right.get(k))) + ")"
        result.append(f"{symbol} {name} {token[0]} {token[1]} {model}{changes}".rstrip())
    return result or [f"~ {name}（来源、注释或其他字段）"]


def research(checkout, stable_ref, dev_ref):
    stable, dev = resolve(checkout, stable_ref), resolve(checkout, dev_ref)
    old_tree, new_tree = (git_tree(checkout, ref, PLUGINS) for ref in (stable, dev))
    docs = git_tree(checkout, dev, BRANDS)
    vendor_dirs = sorted({p.split("/")[3] for p in old_tree.keys() | new_tree.keys()
                          if len(p.split("/")) > 4 and p.split("/")[3] != "Protocol"})
    info = {"repository": REPO, "revision": dev, "stable_tag": stable_ref,
            "stable_revision": stable, "files": {"LICENSE": "LICENSE"}}
    files = {}
    changes = defaultdict(list)
    for path in sorted(p for p in new_tree if p.endswith(".json")):
        local = Path(path).name
        if local in info["files"]:
            raise ValueError(f"Upstream file basename used twice: {local}")
        info["files"][local] = path
    for local, path in info["files"].items():
        data = git(checkout, "show", f"{dev}:{path}")
        files[local] = data
        if local.endswith(".json") and old_tree.get(path) != new_tree[path]:
            previous = git(checkout, "show", f"{stable}:{path}") if path in old_tree else b""
            changes[path.split("/")[3]].extend(differences(local, previous, data))
    for path in sorted(old_tree.keys() - new_tree.keys()):
        if path.endswith(".json"):
            changes[path.split("/")[3]].extend(differences(Path(path).name, git(checkout, "show", f"{stable}:{path}"), b""))
    counts = {}
    for vendor in vendor_dirs:
        prefix = f"{PLUGINS}/{vendor}/"
        paths = [p for p in new_tree.keys() | old_tree.keys() if p.startswith(prefix)]
        counts[vendor] = sum(old_tree.get(p) != new_tree.get(p) for p in paths)
    info["brand_docs"] = sorted(docs)
    files["sources.json"] = dump(info)
    files["COVERAGE.md"] = render(info, files, counts, changes, new_tree).encode()
    return files, info


def render(info, files, counts, changes, new_tree):
    sha, stable = info["revision"], info["stable_revision"]
    base = f"{REPO}/blob/{sha}/"
    lines = ["# Vigil Watt 厂商研究对照", "",
             f"稳定版 [{info['stable_tag']}]({REPO}/releases/tag/{info['stable_tag']}) 与开发版 "
             f"[{sha[:7]}]({REPO}/tree/{sha}) 的[完整差异]({REPO}/compare/{stable}...{sha})。", "",
             "## 上游厂商实现", "", "| 上游模块 | 有变化的文件 | 稳定版至开发版的结构化变动 |",
             "| --- | ---: | --- |"]
    for vendor, count in counts.items():
        events = Counter(t[0] for t in changes[vendor])
        marks = " / ".join(f"{k}{events[k]}" for k in "+~-" if events[k]) or "—"
        path = f"{PLUGINS}/{vendor}"
        rev = sha if any(p.startswith(path + "/") for p in new_tree) else stable
        lines.append(f"| [{vendor}]({REPO}/tree/{rev}/{path}) | {count} | {marks} |")
    lines += ["", "变动符号依次代表新登记、内容修改和移除。以下链接列出厂商的协议、采样和设备范围。", "",
              " · ".join(f"[{Path(p).stem}]({base}{p})" for p in info["brand_docs"]), "",
              "## 型号与接口", "",
              "每行是一条上游身份。插件链接只覆盖链接中注明的接口和连接方式。"
              "共用接收器一栏列出本库声明同一硬件标识的插件数，具体型号由设备认领和插件判断。"
              "“待对照”可用于挑选下一项适配，完整型号列表也可在[本库厂商目录](../../README.md)查找。"]
    maps = json.loads((DATA / "plugin-links.json").read_text(encoding="utf-8"))
    packages = {}
    for channel in ("drivers-preview.json", "drivers.json"):
        packages.update((p["id"], p) for p in json.loads((ROOT / channel).read_text(encoding="utf-8")))
    for local, raw in files.items():
        if not local.endswith(".json") or local == "sources.json":
            continue
        doc = parse(raw)
        if "devices" not in doc and "receivers" not in doc:
            continue
        vendor = local.split("-")[0]
        lines += ["", f"### {vendor}", "", f"[上游固定版本]({base}{info['files'][local]}) · [目录原文]({local})",
                  "", "| 上游型号或通道 | VID:PID | 协议 / 连接方式 | 本库对应 |", "| --- | --- | --- | --- |"]
        for (section, key), row in entries(doc).items():
            if section not in {"devices", "transports", "receivers"}:
                continue
            role = row.get("role") or ("有线" if row.get("wired") else "无线" if row.get("wired") is False else "")
            value = row.get("protocol", row.get("family", ""))
            label = row.get("model") or row.get("familyDisplay") or ("接收器" if section != "devices" else "未命名")
            mapping = maps.get(vendor, {}).get(key)
            if mapping:
                plugin, scope = mapping
                if plugin not in packages:
                    raise ValueError(f"Unknown plugin in plugin-links.json: {plugin}")
                result = f"[{plugin}](../../../drivers/{plugin}) · {scope}"
            else:
                candidates = [p for p in packages.values() if p["device"]["brand"].casefold() == vendor
                              and any(f"{m['vid']:04X}:{m['pid']:04X}" == key for m in p["device"]["matches"])]
                result = f"同标识候选 {len(candidates)} 款" if candidates else "待对照"
            lines.append(f"| {label} | {key} | {' · '.join(v for v in [value, role] if v)} | {result} |")
        if doc.get("officialDevices"):
            lines += ["", f"另有 {len(doc['officialDevices'])} 条限于官方登记范围的 PID，可在目录原文的 officialDevices 中按型号查询。"]
    return "\n".join(lines) + "\n"


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--checkout", type=Path, help="existing Vigil Watt git repository; use its local refs without fetching")
    p.add_argument("--stable-tag", help="stable release tag (required with --checkout)")
    p.add_argument("--dev-ref", help="development commit or local ref, defaults to origin/dev")
    p.add_argument("--vendor", help="only display changes for this upstream module")
    p.add_argument("--write", action="store_true", help="refresh the complete pinned source snapshot and coverage page")
    p.add_argument("--check", action="store_true", help="exit 1 if refresh would change tracked files")
    args = p.parse_args()
    if args.write and (args.vendor or args.check):
        p.error("--write is a full revision update and cannot be combined with --vendor or --check")
    checkout = args.checkout
    stable = args.stable_tag
    dev = args.dev_ref or "origin/dev"
    if checkout is None:
        if not stable:
            with urlopen("https://api.github.com/repos/evren-oct/vigil-watt/releases/latest", timeout=20) as response:
                stable = json.load(response)["tag_name"]
        checkout = ROOT / "work/upstreams/vigil-watt"
        if not checkout.exists():
            checkout.parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(["git", "clone", "--no-checkout", "--filter=blob:none", "--depth=1",
                            "--branch", "dev", REPO, str(checkout)], check=True)
        git(checkout, "fetch", "origin", "+refs/heads/dev:refs/remotes/origin/dev", f"refs/tags/{stable}:refs/tags/{stable}")
    elif not stable:
        p.error("--checkout requires --stable-tag, for example v1.3.1")
    files, info = research(checkout, stable, dev)
    old = json.loads((DATA / "sources.json").read_text(encoding="utf-8"))
    removed = sorted(old["files"].keys() - info["files"].keys())
    if args.vendor and args.vendor.lower() not in {p.split("/")[3].lower() for p in git_tree(checkout, info["revision"], PLUGINS) if len(p.split("/")) > 4}:
        p.error(f"Unknown upstream vendor {args.vendor}")
    changed = False
    for local in sorted(set(files) | set(removed)):
        path = DATA / local
        before = path.read_bytes().replace(b"\r\n", b"\n") if path.is_file() else b""
        after = files.get(local, b"")
        if before == after:
            continue
        source = info["files"].get(local, old["files"].get(local, ""))
        if args.vendor and f"/{args.vendor.lower()}/" not in source.lower():
            continue
        changed = True
        if local.endswith(".json") and local != "sources.json":
            for line in differences(local, before, after):
                print(line)
        else:
            print(f"~ {local}")
        if args.write:
            if local in removed:
                path.unlink()
            else:
                path.write_bytes(after)
    old_sha = old["revision"]
    print(f"stable {stable} · dev {old_sha[:7]} → {info['revision'][:7]}")
    if old_sha != info["revision"]:
        if not args.checkout:
            # A first shallow clone may not contain a formerly pinned dev commit.
            if subprocess.run(["git", "-C", str(checkout), "cat-file", "-e", old_sha + "^{commit}"],
                              stderr=subprocess.DEVNULL).returncode:
                git(checkout, "fetch", "--depth=1", "origin", old_sha)
        args_diff = ["diff", "--name-status", old_sha, info["revision"], "--", PLUGINS, BRANDS]
        for line in git(checkout, *args_diff).decode().splitlines():
            if not args.vendor or args.vendor.casefold() in line.casefold():
                print(line)
                changed = True
    if args.check:
        return int(changed)
    return 0


if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    sys.exit(main())

