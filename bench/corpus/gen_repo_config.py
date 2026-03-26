#!/usr/bin/env python3
"""Generate a per-repo RuboCop config overlay inside the repo directory.

Writes a `.rubocop_corpus.yml` overlay inside <repo_dir> that inherits from
the base config. The `.rubocop`-prefixed name ensures both RuboCop and nitrocop
resolve `base_dir` to the repo directory (not CWD), which is critical for
cop-level Include/Exclude patterns like `db/**/*.rb` to match correctly.

If the repo has file exclusions (from repo_excludes.json), the overlay also
includes those Exclude patterns.

Usage:
    python3 gen_repo_config.py <repo_id> <base_config> <repo_dir>

Always prints the path to the generated overlay (inside repo_dir).
"""
import json
import sys
from pathlib import Path

EXCLUDES_PATH = Path(__file__).parent / "repo_excludes.json"


def gen_repo_config(repo_id: str, base_config: str, repo_dir: str) -> str:
    """Generate overlay config inside repo_dir. Returns path to overlay."""
    abs_base = str(Path(base_config).resolve())
    abs_repo = str(Path(repo_dir).resolve())

    lines = [f"inherit_from: {abs_base}"]

    # Add per-repo file exclusions if any exist.
    excludes: list[str] = []
    if EXCLUDES_PATH.exists():
        with open(EXCLUDES_PATH) as f:
            all_excludes = json.load(f)
        entry = all_excludes.get(repo_id)
        if entry and entry.get("exclude"):
            excludes = entry["exclude"]

    if excludes:
        # RuboCop merges AllCops/Exclude by default (union), so we only need
        # to list the additional excludes here.
        lines += ["", "AllCops:", "  Exclude:"]
        for pattern in excludes:
            lines.append(f'    - "{abs_repo}/{pattern}"')

    overlay_path = Path(abs_repo) / ".rubocop_corpus.yml"
    overlay_path.write_text("\n".join(lines) + "\n")
    return str(overlay_path)


def main():
    if len(sys.argv) != 4:
        print(f"Usage: {sys.argv[0]} <repo_id> <base_config> <repo_dir>", file=sys.stderr)
        sys.exit(1)

    repo_id, base_config, repo_dir = sys.argv[1], sys.argv[2], sys.argv[3]
    print(gen_repo_config(repo_id, base_config, repo_dir))


if __name__ == "__main__":
    main()
