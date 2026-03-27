#!/usr/bin/env python3
"""Validate that a rubocop JSON result only contains files under a repo directory.

Exits 0 if valid, 1 if any file path is outside the expected repo directory.
Used by corpus-oracle.yml to prevent cache poisoning from misconfigured runs.
"""

import json
import sys


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <result.json> <repo_dir>", file=sys.stderr)
        sys.exit(2)

    result_path, repo_dir = sys.argv[1], sys.argv[2]
    prefix = repo_dir.rstrip("/") + "/"

    with open(result_path) as f:
        data = json.load(f)

    for fobj in data.get("files", []):
        path = fobj.get("path", "")
        if not path.startswith(prefix):
            print(f"POISONED: {path} not under {prefix}", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
