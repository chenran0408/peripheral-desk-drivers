#!/usr/bin/env python3
"""Check package metadata, the custom-handler starter and local documentation links."""
import json
from pathlib import Path
import re
import struct
import tomllib

ROOT = Path(__file__).resolve().parents[1]

def metadata(item):
    required = {"id", "name", "version", "description", "abi", "dll", "device", "core_min", "core_max_exclusive", "channel", "capabilities", "changelog"}
    if not required.issubset(item) or item["abi"] != 1 or item["dll"] != "driver.dll":
        raise ValueError("Missing package contract fields")
    if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", item["id"]):
        raise ValueError("Unsafe package id")
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?", item["version"]):
        raise ValueError("Invalid independently versioned package")
    if not re.fullmatch(r"0\.4\.(0|[1-9][0-9]*)", item["core_min"]) or item["core_max_exclusive"] != "0.5.0":
        raise ValueError("ABI 1 packages must retain the reviewed 0.4.x core range")
    if item["channel"] not in ("stable", "preview") or not item["changelog"]:
        raise ValueError("Missing channel or user-facing changelog")
    device = item["device"]
    if device["key"] != item["id"] or device["image"] != "device.png" or device["kind"] not in ("mouse", "keyboard", "headset", "accessory"):
        raise ValueError("Device identity/kind mismatch")
    if not device["matches"]:
        raise ValueError("Hardware matching rules are required")
    for match in device["matches"]:
        for field in ("vid", "pid"):
            if type(match[field]) is not int or not 0 < match[field] <= 65535:
                raise ValueError("VID/PID must be explicit nonzero u16 values")
        for field in ("usage_page", "usage", "feature_len", "input_len", "output_len"):
            if field in match and (type(match[field]) is not int or not 0 <= match[field] <= 65535):
                raise ValueError("Invalid matcher " + field)
    for advanced in device.get("advanced", []):
        if not all(advanced.get(key) for key in ("key", "title", "hint", "choices")):
            raise ValueError("Incomplete advanced setting")
        values = []
        for value, label in advanced["choices"]:
            if type(value) is not int or not 0 <= value <= 65535 or not isinstance(label, str) or not label:
                raise ValueError("Invalid advanced choice")
            values.append(value)
        if len(values) != len(set(values)):
            raise ValueError("Duplicate advanced choice")

def main():
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]
    for filename, channel in (("drivers.json", "stable"), ("drivers-preview.json", "preview")):
        items = json.loads((ROOT / filename).read_text(encoding="utf-8"))
        if len({item["id"] for item in items}) != len(items):
            raise ValueError("Duplicate package id in " + filename)
        for item in items:
            metadata(item)
            if item["channel"] != channel:
                raise ValueError("Package is in the wrong channel metadata")
            package = ROOT / "drivers" / item["id"]
            crate = tomllib.loads((package / "Cargo.toml").read_text(encoding="utf-8"))["package"]
            version = crate.get("version")
            if version == {"workspace": True}:
                version = workspace["package"]["version"]
            if crate.get("name") != item["id"] + "-driver" or version != item["version"]:
                raise ValueError("Crate name/version disagrees with catalog: " + item["id"])
            if "drivers/" + item["id"] not in workspace["members"]:
                raise ValueError("Driver is missing from workspace members: " + item["id"])
            source = (package / "src/lib.rs").read_text(encoding="utf-8")
            if not re.search(r'export_driver!\s*\(\s*"' + re.escape(item["id"]) + r'"', source):
                raise ValueError("Wrapper key does not match its metadata")
            if not (ROOT / "assets" / (item["id"] + ".png")).read_bytes().startswith(b"\x89PNG\r\n\x1a\n"):
                raise ValueError("Device image is not a PNG")
    template = ROOT / "templates/device"
    metadata(json.loads((template / "metadata.json").read_text(encoding="utf-8")))
    if 'export_driver!("example-device", crate::adapter::request)' not in (template / "lib.rs.template").read_text(encoding="utf-8"):
        raise ValueError("Starter must dispatch to its custom handler")
    for file in template.glob("*.template"):
        if re.search(r"\b(?:todo|unimplemented)!", file.read_text(encoding="utf-8")):
            raise ValueError("Starter contains an aborting placeholder")
    blob = (ROOT / "assets/catalog-public-key.blob").read_bytes()
    magic, bits, exponent, modulus, p, q = struct.unpack("<6I", blob[:24])
    if magic != 0x31415352 or bits < 2048 or modulus * 8 != bits or p or q or len(blob) != 24 + exponent + modulus:
        raise ValueError("Public key must be an RSA public blob, never a private key")
    for document in list(ROOT.glob("*.md")) + list((ROOT / "docs").glob("*.md")) + list(template.glob("*.md")):
        for link in re.findall(r"\]\(([^)]+)\)", document.read_text(encoding="utf-8")):
            if ":" not in link and not link.startswith("#") and not (document.parent / link.split("#", 1)[0]).exists():
                raise ValueError(f"Broken documentation link in {document.name}: {link}")
    print("Checked package metadata/crate versions, workspace membership, custom-handler starter, public key and documentation links")

if __name__ == "__main__":
    main()
