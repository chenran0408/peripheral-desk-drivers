"""ABI 1 release-family minima and optional numbered preview requirements."""
import re

def minimum_order(text):
    match = re.fullmatch(r"0\.4\.(0|[1-9][0-9]*)(?:-preview\.(0|[1-9][0-9]*))?", text)
    if not match:
        raise ValueError(f"Unsupported minimum core version: {text}")
    # A numeric family includes its previews for ABI 1 compatibility.
    return int(match[1]), int(match[2]) if match[2] is not None else -1

def check_range(entry):
    minimum = minimum_order(entry["core_min"])
    if entry["core_max_exclusive"] != "0.5.0":
        raise ValueError("ABI 1 packages require the reviewed 0.4.x core range")
    return minimum
