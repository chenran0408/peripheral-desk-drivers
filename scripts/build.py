#!/usr/bin/env python3
"""Build four Windows x64 ABI 1 DLL packages and their trusted catalog."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BASE_URL = "https://github.com/chenran0408/peripheral-desk-drivers/releases/download"

def sha256(data):
    return hashlib.sha256(data).hexdigest()

def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", default="x86_64-pc-windows-gnu")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable path")
    parser.add_argument("--toolchain", help="Optional rustup toolchain, e.g. stable-x86_64-pc-windows-gnu")
    parser.add_argument("--metadata", type=Path, default=ROOT / "drivers.json", help="Package metadata file; preview packages use drivers-preview.json")
    parser.add_argument("--package", action="append", dest="packages", help="Build/package only this driver id; repeat for several")
    parser.add_argument("--skip-build", action="store_true", help="Package already-built release DLLs")
    parser.add_argument("--out", type=Path, default=ROOT / "dist")
    parser.add_argument("--release", action="store_true", help="Require a pinned Git core dependency; reject development path dependencies")
    parser.add_argument("--release-tag", help="Release that will receive every generated ZIP (default: v<workspace version>)")
    args = parser.parse_args()
    if not args.target.startswith("x86_64-pc-windows-"):
        parser.error("ABI 1 downloads currently target Windows x64 only")
    manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    workspace_version = re.search(r'^version\s*=\s*"([^"]+)"', manifest, re.M)
    release_tag = args.release_tag or ("v" + workspace_version.group(1) if workspace_version else "")
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+(?:[-.][a-zA-Z0-9.-]+)?", release_tag):
        parser.error("Release tag must be v<semver> with an optional preview suffix")
    core = re.search(r'^peripheral-desk\s*=\s*\{([^\n]+)\}', manifest, re.M)
    if not core:
        parser.error("Cannot locate the core workspace dependency")
    revision = re.search(r'\brev\s*=\s*"([0-9a-fA-F]{40})"', core.group(1))
    if args.release and ("path" in core.group(1) or not revision):
        parser.error("Release builds require the reviewed core Git URL and an immutable 40-digit rev")
    items = json.loads(args.metadata.read_text(encoding="utf-8"))
    if args.packages:
        missing = set(args.packages) - {item["id"] for item in items}
        if missing:
            parser.error("Unknown package ids: " + ", ".join(sorted(missing)))
        items = [item for item in items if item["id"] in args.packages]
    if not items:
        parser.error("No drivers selected")
    if len({item["id"] for item in items}) != len(items):
        parser.error("Duplicate driver ids")
    if not args.skip_build:
        # Separate invocations keep per-device FFI features from unifying in
        # a multi-package build and retaining unrelated protocol handlers.
        for item in items:
            command = [args.cargo]
            if args.toolchain:
                command.append("+" + args.toolchain)
            command += ["build", "--release", "--target", args.target,
                        "--package", item["id"] + "-driver"]
            if args.release:
                command.append("--locked")
            subprocess.run(command, cwd=ROOT, check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    if not target.is_absolute():
        target = ROOT / target
    args.out.mkdir(parents=True, exist_ok=True)
    entries, checksums = [], []
    for item in items:
        driver_id, version = item["id"], item["version"]
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", driver_id):
            parser.error("Driver id must be a safe lowercase package identifier")
        if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-.a-zA-Z0-9]*)", version):
            parser.error("Invalid version")
        dll_path = target / args.target / "release" / (driver_id.replace("-", "_") + "_driver.dll")
        dll = dll_path.read_bytes()
        if dll[:2] != b"MZ":
            parser.error(str(dll_path) + " is not a Windows PE DLL")
        manifest = {"schema": 1, "abi": 1, "dll": "driver.dll", **item, "dll_sha256": sha256(dll)}
        # This manifest intentionally excludes the ZIP hash, URL and size.
        for field in ["sha256", "package_url", "download_bytes"]:
            manifest.pop(field, None)
        name = f"{driver_id}-{version}-windows-x64.zip"
        package = args.out / name
        payloads = {
            "driver.dll": dll,
            "device.png": (ROOT / "assets" / (driver_id + ".png")).read_bytes(),
            "manifest.json": (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8"),
            "LICENSE": (ROOT / "LICENSE").read_bytes(),
            "README.txt": (f"{item['name']} {version}\n在 Peripheral Desk 的驱动商店安装此包。\n请勿直接运行或注册 driver.dll。\n源码：https://github.com/chenran0408/peripheral-desk-drivers\n" + ("G2 Pro 音效需要已安装的 HECATE / C-Media 官方组件；包内不附带厂商驱动。\n" if driver_id == "hecate-g2-pro" else "")).encode("utf-8"),
        }
        payloads["SOURCE.txt"] = ("Driver wrapper source: https://github.com/chenran0408/peripheral-desk-drivers\nCore source: https://github.com/chenran0408/peripheral-desk/tree/" + (revision.group(1) if revision else "DEVELOPMENT-UNPINNED") + "\n").encode("utf-8")
        # Ship model-specific evidence/limitations alongside the binary.
        evidence = {"ajazz-aj179-pro": "AJAZZ-AJ179-PRO.md", "mchose-magdock": "MCHOSE-MAGDOCK.md", "razer-viper-v2-pro-wired": "RAZER-VIPER-V2-PRO.md"}.get(driver_id)
        if evidence:
            payloads["DEVICE_SUPPORT.md"] = (ROOT / "docs" / evidence).read_bytes()
        payloads["ASSET_SOURCES.md"] = (ROOT / "ASSET_SOURCES.md").read_bytes()
        for license_path in sorted((ROOT / "THIRD_PARTY_LICENSES").rglob("*")):
            if license_path.is_file():
                payloads[license_path.relative_to(ROOT).as_posix()] = license_path.read_bytes()
        with zipfile.ZipFile(package, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for filename, data in payloads.items():
                info = zipfile.ZipInfo(filename, date_time=(1980, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o644 << 16
                archive.writestr(info, data)
        blob = package.read_bytes()
        entry = {**item, "abi": 1, "dll": "driver.dll", "dll_sha256": sha256(dll), "download_bytes": len(blob), "package_url": f"{BASE_URL}/{release_tag}/{name}", "sha256": sha256(blob)}
        entries.append(entry)
        checksums.append(f"{entry['sha256']}  {name}")
        print(f"{driver_id}: DLL {entry['dll_sha256']}, ZIP {entry['sha256']}, {len(blob)} bytes")
    stable = [entry for entry in entries if entry["channel"] == "stable"]
    preview = [entry for entry in entries if entry["channel"] == "preview"]
    write_json(args.out / "catalog.json", {"schema": 1, "channel": "stable", "drivers": stable})
    write_json(args.out / "catalog-stable.json", {"schema": 1, "channel": "stable", "drivers": stable})
    write_json(args.out / "catalog-preview.json", {"schema": 1, "channel": "preview", "drivers": preview})
    (args.out / "SHA256SUMS.txt").write_text("\n".join(checksums) + "\n", encoding="utf-8")
    write_json(args.out / "build-info.json", {"target": args.target, "abi": 1, "release_tag": release_tag, "core_revision": revision.group(1) if revision else None, "development_path_dependency": revision is None})
    print("Built catalog:", args.out / "catalog.json")

if __name__ == "__main__":
    main()
