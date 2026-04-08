#!/usr/bin/env python3
"""Merge infobox-extracted aliases into the main aliases.json file."""

import json
import sys
from pathlib import Path


def main():
    aliases_path = Path("data/aliases.json")
    extracted_path = Path("/tmp/extracted_aliases.json")

    # Load existing aliases
    if aliases_path.exists():
        with open(aliases_path, "r", encoding="utf-8") as f:
            existing = json.load(f)
    else:
        existing = {}

    # Load extracted aliases
    if not extracted_path.exists() or extracted_path.stat().st_size == 0:
        print("No extracted aliases to merge")
        return

    with open(extracted_path, "r", encoding="utf-8") as f:
        extracted = json.load(f)

    # Merge
    added = 0
    for key, value in extracted.items():
        if key not in existing:
            existing[key] = value
            added += 1

    # Save
    with open(aliases_path, "w", encoding="utf-8") as f:
        json.dump(existing, f, ensure_ascii=False, indent=2)

    print(f"Added {added} infobox aliases")


if __name__ == "__main__":
    main()
