"""Exercise signed release ZIPs through the actual Windows host, outside user data."""
import argparse
import hashlib
import json
import subprocess
import tempfile
import zipfile
from pathlib import Path

CORE = "chenran0408/peripheral-desk"
DRIVERS = CORE + "-drivers"


def fetch(repo, tag, directory, *names):
    directory.mkdir(parents=True, exist_ok=True)
    command = ["gh", "release", "download", tag, "--repo", repo, "--dir", str(directory)]
    for name in names:
        command.extend(["--pattern", name])
    subprocess.run(command, check=True)


def core_binary(work, tag):
    name = f"PeripheralDesk-{tag.removeprefix('v')}-windows-x64.zip"
    fetch(CORE, tag, work, name, "SHA256SUMS.txt")
    sums = dict(line.split(maxsplit=1)[::-1] for line in
                (work / "SHA256SUMS.txt").read_text(encoding="utf-8-sig").splitlines() if line.strip())
    archive = work / name
    if hashlib.sha256(archive.read_bytes()).hexdigest() != sums[name].lower():
        raise ValueError("Host archive differs from its released checksum")
    exe = work / "PeripheralDesk.exe"
    with zipfile.ZipFile(archive) as z:
        exe.write_bytes(z.read("PeripheralDesk.exe"))
    return exe


def cli(exe, op, value):
    report = exe.parent / "result.json"
    if report.exists():
        report.unlink()
    run = subprocess.run([str(exe), "--driver", op, str(value), str(report)], timeout=90)
    result = json.loads(report.read_text(encoding="utf-8")) if report.exists() else {}
    if run.returncode or not result.get("ok"):
        raise ValueError(f"Host {op} failed for {value}: {result.get('error', run.returncode)}")
    return result["value"]


def channel(exe, name):
    folder = exe.parent / "data/drivers"
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "channel.txt").write_text(name, encoding="utf-8")


def check(folder, work, host_tag):
    exe = core_binary(work, host_tag)
    previous = {}
    for name in ("stable", "preview"):
        catalog_name = f"catalog-{name}.json"
        fetch(DRIVERS, f"catalog-{name}", work / name, catalog_name, catalog_name + ".sig")
        channel(exe, name)
        for item in cli(exe, "import-catalog", work / name / catalog_name):
            previous[item["id"]] = (name, item)
    for name in ("stable", "preview"):
        catalog = folder / f"catalog-{name}.json"
        channel(exe, name)
        for item in cli(exe, "import-catalog", catalog):
            key = item["id"]
            prior = previous.get(key)
            earlier = prior and prior[1]["version"] != item["version"]
            data = exe.parent / "data/devices" / key
            data.mkdir(parents=True, exist_ok=True)
            marker = data / "community-test.json"
            marker.write_bytes(b'{"unknown_field":[1,2,3]}')
            original = marker.read_bytes()
            if earlier:
                old_channel, old = prior
                channel(exe, old_channel)
                cli(exe, "import-catalog", work / old_channel / f"catalog-{old_channel}.json")
                installed = cli(exe, "install", key)
                if installed["version"] != old["version"]:
                    raise ValueError("Unexpected prior version")
            channel(exe, name)
            cli(exe, "import-catalog", catalog)
            archive = folder / item["package_url"].rsplit("/", 1)[-1]
            if cli(exe, "offline", archive)["version"] != item["version"]:
                raise ValueError("Host failed to select the candidate version")
            if earlier and cli(exe, "rollback", key)["version"] != old["version"]:
                raise ValueError("Host failed to return to the previous version")
            cli(exe, "uninstall", key)
            if marker.read_bytes() != original:
                raise ValueError("Updating or uninstalling changed user configuration")
            print(key, item["version"], "install / configuration retained", "/ rollback" if earlier else "")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog_directory", type=Path)
    parser.add_argument("--host-tag", required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="pd-host-") as temp:
        check(args.catalog_directory.resolve(), Path(temp), args.host_tag)
