#!/usr/bin/env python3
"""Validate corpus manifests for duplicates and structural issues.

Checks manifest.jsonl and manifest_extended.jsonl (if present), including
cross-file duplicate detection.

Usage:
    python3 bench/corpus/validate_manifest.py
"""
import json
import sys
from pathlib import Path

CORPUS_DIR = Path(__file__).parent
MANIFESTS = [
    CORPUS_DIR / "manifest.jsonl",
    CORPUS_DIR / "manifest_extended.jsonl",
]


def main() -> int:
    errors = 0
    # Global dedup across all manifest files
    all_ids: dict[str, str] = {}   # id -> "file:line"
    all_urls: dict[str, str] = {}  # normalized url -> "file:line"

    for manifest_path in MANIFESTS:
        if not manifest_path.exists():
            continue

        label = manifest_path.name
        entries = []

        for i, line in enumerate(manifest_path.read_text().splitlines(), 1):
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"ERROR: {label}:{i}: invalid JSON: {e}")
                errors += 1
                continue

            for key in ("id", "repo_url", "sha"):
                if key not in entry:
                    print(f"ERROR: {label}:{i}: missing required key '{key}'")
                    errors += 1

            entries.append((i, entry))

        # Check duplicate IDs (within file and across files)
        for lineno, entry in entries:
            rid = entry.get("id", "")
            loc = f"{label}:{lineno}"
            if rid in all_ids:
                print(f"ERROR: duplicate id '{rid}' at {all_ids[rid]} and {loc}")
                errors += 1
            else:
                all_ids[rid] = loc

        # Check duplicate repo URLs (within file and across files)
        for lineno, entry in entries:
            url = entry.get("repo_url", "").rstrip("/").lower()
            loc = f"{label}:{lineno}"
            if url in all_urls:
                print(f"ERROR: duplicate repo_url '{url}' at {all_urls[url]} and {loc}")
                errors += 1
            else:
                all_urls[url] = loc

        print(f"{label}: {len(entries)} entries", file=sys.stderr)

    total = len(all_ids)
    if errors:
        print(f"\nFAILED: {errors} error(s) in {total} total entries")
        return 1

    print(f"OK: {total} total entries across {sum(1 for p in MANIFESTS if p.exists())} file(s), no duplicates")
    return 0


if __name__ == "__main__":
    sys.exit(main())
