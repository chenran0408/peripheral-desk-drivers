#!/usr/bin/env python3
"""Validate release catalogs/ZIP hashes/manifests, optionally check native ABI without HID I/O."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import zipfile

REQUIRED = ["id", "name", "version", "description", "abi", "dll", "dll_sha256", "device", "core_min", "core_max_exclusive", "channel", "capabilities", "changelog"]
HECATE_COMPONENTS = {
    "sdk": "components/sdk/osConfLib.dll",
    "driver-inf": "components/windows/HECATE_HD_AUDIO_DEVICE-Driver-V11.2.1.0/HECATE_HD_AUDIO_DEVICE.Inf",
    "driver-cat": "components/windows/HECATE_HD_AUDIO_DEVICE-Driver-V11.2.1.0/HECATE_HD_AUDIO_DEVICE.cat",
    "driver-sys": "components/windows/HECATE_HD_AUDIO_DEVICE-Driver-V11.2.1.0/x64/HECATE_HD_AUDIO_DEVICE.sys",
    "extension-inf": "components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/01_EXTENSION/HECATE_HD_AUDIO_DEVICE_Ext.inf",
    "extension-cat": "components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/01_EXTENSION/CMExt.cat",
    "apo-inf": "components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/02_APO/HECATE_HD_AUDIO_DEVICE_APO.inf",
    "apo-cat": "components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/02_APO/CMAPO.cat",
    "apo-engine": "components/windows/CM6646X1_HECATE_HD_AUDIO_DEVICE_APO_V2.1.0.2/02_APO/CmApoIce.dll",
}
def digest(data):
    return hashlib.sha256(data).hexdigest()

def abi_dll(path):
    if os.name != "nt" or ctypes.sizeof(ctypes.c_void_p) != 8:
        raise ValueError("Native ABI check requires 64-bit Windows Python")
    class Buffer(ctypes.Structure):
        _fields_ = [("ptr", ctypes.c_void_p), ("len", ctypes.c_size_t)]
    library = ctypes.CDLL(str(path.resolve()))
    try:
        library.pd_driver_abi.argtypes = []
        library.pd_driver_abi.restype = ctypes.c_uint32
        if library.pd_driver_abi() != 1:
            raise ValueError("Unexpected ABI number")
        library.pd_driver_request.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
        library.pd_driver_request.restype = Buffer
        library.pd_driver_free.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
        library.pd_driver_free.restype = None
        # Invalid UTF-8 is rejected before protocol dispatch. Never read or write HID in CI.
        invalid = ctypes.create_string_buffer(b"\xff")
        result = library.pd_driver_request(invalid, 1)
        try:
            if not result.ptr or result.len > 1048576:
                raise ValueError("Invalid returned Buffer")
            reply = json.loads(ctypes.string_at(result.ptr, result.len).decode("utf-8"))
            if reply.get("ok") is not False or not reply.get("error"):
                raise ValueError("FFI validation did not return an error envelope")
        finally:
            library.pd_driver_free(result.ptr, result.len)
    finally:
        # Release the module before deleting its temporary file on Windows.
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel32.FreeLibrary.argtypes = [ctypes.c_void_p]
        kernel32.FreeLibrary.restype = ctypes.c_int
        if not kernel32.FreeLibrary(library._handle):
            raise ctypes.WinError(ctypes.get_last_error())

def abi_check(dll, directory):
    # A malformed PR DLL may hang or crash even on invalid UTF-8. Isolate the
    # native probe and bound it before attempting to remove the loaded file.
    with tempfile.TemporaryDirectory(prefix="pd-abi-", dir=directory) as temporary:
        path = Path(temporary) / "driver.dll"
        path.write_bytes(dll)
        subprocess.run([sys.executable, str(Path(__file__).resolve()), "--abi-dll", str(path)], check=True, timeout=30)

def verify_catalog(path, abi, temporary, release_tag=None):
    catalog = json.loads(path.read_text(encoding="utf-8"))
    if catalog.get("schema") != 1 or catalog.get("channel") not in ["stable", "preview"]:
        raise ValueError("Catalog schema/channel mismatch")
    ids = set()
    for entry in catalog["drivers"]:
        if entry["id"] in ids:
            raise ValueError("Duplicate catalog driver id")
        ids.add(entry["id"])
        if entry["channel"] != catalog["channel"] or entry["abi"] != 1:
            raise ValueError("Driver channel or ABI mismatch")
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", entry["id"]):
            raise ValueError("Unsafe driver id")
        if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?", entry["version"]):
            raise ValueError("Invalid independently versioned driver package")
        if entry["device"]["key"] != entry["id"] or entry["device"]["image"] != "device.png":
            raise ValueError("Device identity/image mismatch")
        declared = entry["device"].get("components", [])
        if entry["id"] == "hecate-g2-pro" and declared:
            if not isinstance(declared, list) or len(declared) != len(HECATE_COMPONENTS):
                raise ValueError("G2 Pro package must declare the reviewed vendor component set")
            component_paths = set()
            for component in declared:
                component_name = component.get("name") if isinstance(component, dict) else None
                if (not isinstance(component, dict)
                        or not isinstance(component_name, str)
                        or component_name not in HECATE_COMPONENTS
                        or component.get("path") != HECATE_COMPONENTS[component_name]
                        or not isinstance(component.get("sha256"), str)
                        or not re.fullmatch(r"[0-9a-f]{64}", component["sha256"])):
                    raise ValueError("Invalid or unreviewed G2 Pro component declaration")
                component_paths.add(component["path"])
            if len(component_paths) != len(HECATE_COMPONENTS):
                raise ValueError("Duplicate or missing G2 Pro component declaration")
        elif declared:
            raise ValueError("Only the G2 Pro package may include vendor components")
        if not re.fullmatch(r"0\.4\.(0|[1-9][0-9]*)", entry["core_min"]) or entry["core_max_exclusive"] != "0.5.0":
            raise ValueError("Update verifier's supported core range before changing the ABI contract")
        if entry["dll"] != "driver.dll" or not entry["device"]["matches"]:
            raise ValueError("Missing DLL or hardware matcher")
        filename = entry["package_url"].rsplit("/", 1)[-1]
        expected = f"{entry['id']}-{entry['version']}-windows-x64.zip"
        url = re.fullmatch(r"https://github\.com/chenran0408/peripheral-desk-drivers/releases/download/(v[0-9]+\.[0-9]+\.[0-9]+(?:[-.][a-zA-Z0-9.-]+)?)/([^/]+)", entry["package_url"])
        if filename != expected or not url or (release_tag and url.group(1) != release_tag):
            raise ValueError("Package URL does not match release identity")
        package = path.parent / filename
        blob = package.read_bytes()
        if len(blob) != entry["download_bytes"] or digest(blob) != entry["sha256"]:
            raise ValueError("ZIP size/hash mismatch: " + filename)
        with zipfile.ZipFile(package) as archive:
            names = archive.namelist()
            if len(blob) > 64 * 1024 * 1024 or len(names) > 128 or sum(info.file_size for info in archive.infolist()) > 128 * 1024 * 1024:
                raise ValueError("Package exceeds desktop installer limits")
            if len(set(names)) != len(names) or not {"driver.dll", "device.png", "manifest.json", "LICENSE"}.issubset(names):
                raise ValueError("Missing or duplicate package files")
            if any(name.startswith(("/", "\\")) or ".." in name.replace("\\", "/").split("/") for name in names):
                raise ValueError("Unsafe ZIP entry")
            declared_component_paths = {component["path"] for component in declared}
            if entry["id"] == "hecate-g2-pro" and declared:
                packaged_components = {name for name in names if name.startswith("components/")}
                if packaged_components != declared_component_paths:
                    raise ValueError("G2 Pro package component files do not match its declarations")
                for component in declared:
                    if digest(archive.read(component["path"])) != component["sha256"]:
                        raise ValueError("G2 Pro component hash mismatch: " + component["path"])
            elif any(name.startswith("components/") for name in names):
                raise ValueError("Only the G2 Pro package may contain a components directory")
            allowed_native = declared_component_paths if entry["id"] == "hecate-g2-pro" else set()
            if any(name.lower().endswith((".exe", ".sys", ".msi", ".dll"))
                   and name != "driver.dll" and name not in allowed_native for name in names):
                raise ValueError("Device package contains an undeclared native binary")
            if archive.getinfo("manifest.json").file_size > 256 * 1024:
                raise ValueError("Manifest exceeds desktop installer limit")
            manifest = json.loads(archive.read("manifest.json").decode("utf-8"))
            if manifest.get("schema") != 1:
                raise ValueError("Manifest schema mismatch")
            for field in REQUIRED:
                if manifest.get(field) != entry.get(field):
                    raise ValueError("Catalog/manifest disagree on " + field)
            if any(field in manifest for field in ["sha256", "download_bytes", "package_url"]):
                raise ValueError("ZIP manifest must not reference its own package hash")
            dll = archive.read("driver.dll")
            if digest(dll) != entry["dll_sha256"]:
                raise ValueError("DLL hash mismatch")
            if abi:
                abi_check(dll, temporary)
        print("Verified", entry["id"], entry["version"], entry["channel"])

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog", type=Path, nargs="?")
    parser.add_argument("--abi-check", action="store_true")
    parser.add_argument("--abi-dll", type=Path, help=argparse.SUPPRESS)
    parser.add_argument("--release-tag", help="Require every package URL to point at this actual release")
    parser.add_argument("--temp-dir", type=Path)
    args = parser.parse_args()
    if args.abi_dll:
        abi_dll(args.abi_dll)
        return
    if not args.catalog:
        parser.error("catalog is required")
    if args.temp_dir:
        args.temp_dir.mkdir(parents=True, exist_ok=True)
    verify_catalog(args.catalog, args.abi_check, args.temp_dir, args.release_tag)

if __name__ == "__main__":
    main()
