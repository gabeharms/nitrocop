#!/usr/bin/env python3
"""Validate corpus manifest.jsonl for duplicates and structural issues.

Usage:
    python3 bench/corpus/validate_manifest.py
"""
import json
import sys
from pathlib import Path

MANIFEST = Path(__file__).parent / "manifest.jsonl"


def main() -> int:
    errors = 0
    entries = []

    for i, line in enumerate(MANIFEST.read_text().splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError as e:
            print(f"ERROR: line {i}: invalid JSON: {e}")
            errors += 1
            continue

        for key in ("id", "repo_url", "sha"):
            if key not in entry:
                print(f"ERROR: line {i}: missing required key '{key}'")
                errors += 1

        entries.append((i, entry))

    # Check duplicate IDs
    ids = {}
    for lineno, entry in entries:
        rid = entry.get("id", "")
        if rid in ids:
            print(f"ERROR: duplicate id '{rid}' on lines {ids[rid]} and {lineno}")
            errors += 1
        else:
            ids[rid] = lineno

    # Check duplicate repo URLs (normalized)
    urls = {}
    for lineno, entry in entries:
        url = entry.get("repo_url", "").rstrip("/").lower()
        if url in urls:
            print(f"ERROR: duplicate repo_url '{url}' on lines {urls[url]} and {lineno}")
            errors += 1
        else:
            urls[url] = lineno

    total = len(entries)
    if errors:
        print(f"\nFAILED: {errors} error(s) in {total} entries")
        return 1

    print(f"OK: {total} entries, no duplicates")
    return 0


if __name__ == "__main__":
    sys.exit(main())
