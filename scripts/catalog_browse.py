"""Add or verify store search fields in an unsigned staging copy of a catalog."""
import argparse
import json
from pathlib import Path

from vendor_catalog import CATALOG, load_json, vendor_key


def load_browse_data():
    aliases = load_json(CATALOG / "vendor-aliases.json")
    families = {}
    for vendor in load_json(CATALOG / "vendors.json")["vendors"]:
        for family in vendor["families"]:
            if family["basis"] == "documented":
                for model in family["models"]:
                    families[vendor["id"], model["channel"], model["id"]] = family["name"]
    return aliases, families


def browse_metadata(entry, aliases, families):
    brand = vendor_key(entry["device"].get("brand", ""))
    return {
        "vendor_aliases": aliases.get(brand, []),
        "family": families.get((brand, entry["channel"], entry["id"]), ""),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="refresh a staging catalog and print the number of changed entries")
    mode.add_argument("--check", action="store_true", help="verify store fields against the checked-out vendor research")
    parser.add_argument("catalog", type=Path)
    args = parser.parse_args()
    catalog = json.loads(args.catalog.read_bytes())
    aliases, families = load_browse_data()
    changed = []
    for entry in catalog["drivers"]:
        expected = browse_metadata(entry, aliases, families)
        browse = entry.get("browse") or {}
        if any(browse.get(key) != value for key, value in expected.items()):
            changed.append(entry["id"])
            entry["browse"] = {**browse, **expected}
    if args.check and changed:
        parser.exit(1, "Outdated store fields: " + ", ".join(changed) + "\n")
    if args.write and changed:
        text = json.dumps(catalog, ensure_ascii=False, separators=(",", ":")) + "\n"
        args.catalog.write_text(text, encoding="utf-8", newline="\n")
    if args.catalog.stat().st_size > 2 * 1024 * 1024:
        parser.exit(1, "Catalog exceeds the host's 2 MiB size limit.\n")
    print(len(changed) if args.write else f"Verified store fields for {len(catalog['drivers'])} entries")


if __name__ == "__main__":
    main()
