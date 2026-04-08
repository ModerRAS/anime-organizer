#!/usr/bin/env python3
"""Create GitHub issues for uncertain alias match proposals."""

import json
import subprocess
import sys
from pathlib import Path


def create_issue(title: str, body: str) -> bool:
    result = subprocess.run(
        [
            "gh",
            "issue",
            "create",
            "--title",
            title,
            "--body",
            body,
            "--label",
            "alias-request",
        ],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


def main():
    uncertain_path = Path("/tmp/uncertain.json")

    if not uncertain_path.exists() or uncertain_path.stat().st_size == 0:
        print("No uncertain proposals found")
        return

    with open(uncertain_path, "r", encoding="utf-8") as f:
        proposals = json.load(f)

    issue_count = 0
    for p in proposals:
        fan = p.get("fan_translation", "Unknown")
        entry = p.get("alias_entry", {})
        bgm_id = entry.get("bangumi_id", 0)
        name = entry.get("name", "")
        confidence = p.get("confidence", "low")
        reasoning = p.get("reasoning", "")

        body_lines = [
            "## Anime Information",
            "",
            "| Field | Value |",
            "|-------|-------|",
            f"| **Bangumi ID** | {bgm_id} |",
            f"| **Bangumi Name** | {name} |",
            f"| **Fan Translation** | {fan} |",
            "",
            "## LLM Analysis",
            "",
            f"**Confidence**: {confidence}",
            f"**Reasoning**: {reasoning}",
            "",
            "## Proposed Alias Entry",
            "",
            "```json",
            f'{{"{fan}": {{"bangumi_id": {bgm_id}, "name": "{name}", "tmdb_id": null, "anidb_id": null}}}}',
            "```",
            "",
            "## User Action Required",
            "",
            "Reply with:",
            "- `confirm` - approve as-is",
            "- `correct: {...}` - provide correction",
            "- `reject` - discard",
        ]
        body = "\n".join(body_lines)
        title = f"[Alias Request] {fan} -> {name} (bgm:{bgm_id})"

        if create_issue(title, body):
            issue_count += 1
        else:
            print(f"Failed to create issue for: {title}")

    print(f"Created {issue_count} alias request issues")


if __name__ == "__main__":
    main()
