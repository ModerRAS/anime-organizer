#!/usr/bin/env python3
"""Update aliases.json with confident LLM-match proposals."""

import json
import sys
from pathlib import Path


def main():
    aliases_path = Path("data/aliases.json")
    confident_path = Path("/tmp/confident.json")

    if not aliases_path.exists():
        print("aliases.json not found, skipping update")
        return

    if not confident_path.exists() or confident_path.stat().st_size == 0:
        print("No confident proposals to merge")
        return

    with open(aliases_path, "r", encoding="utf-8") as f:
        aliases = json.load(f)

    with open(confident_path, "r", encoding="utf-8") as f:
        proposals = json.load(f)

    added_count = 0
    for p in proposals:
        key = p.get("fan_translation", "")
        if key and key not in aliases:
            entry = p.get("alias_entry", {})
            aliases[key] = {
                "bangumi_id": entry.get("bangumi_id"),
                "name": entry.get("name"),
                "tmdb_id": entry.get("tmdb_id"),
                "anidb_id": entry.get("anidb_id"),
            }
            added_count += 1

    with open(aliases_path, "w", encoding="utf-8") as f:
        json.dump(aliases, f, ensure_ascii=False, indent=2)

    print(f"Added {added_count} new aliases")


if __name__ == "__main__":
    main()
