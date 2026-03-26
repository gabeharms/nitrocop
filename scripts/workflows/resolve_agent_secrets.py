#!/usr/bin/env python3
"""Write backend-specific secrets to individual files for the run-agent action.

Reads resolve_backend.py output to determine which secrets the backend
needs, looks them up from _SECRET_* env vars, and writes each to its
own file.  The run-agent action sources these files in step 2 so that
only the needed secrets are in the agent's environment.

Usage:
    python3 resolve_agent_secrets.py <backend_env_file> <secrets_dir>
"""

from __future__ import annotations

import os
import sys
from pathlib import Path


def resolve(backend_env_path: str, secrets_dir: str) -> None:
    out = Path(secrets_dir)
    out.mkdir(exist_ok=True)

    for line in open(backend_env_path):
        if not line.startswith("secret_"):
            continue
        key, _, target = line.strip().partition("=")
        source_name = key[len("secret_"):]
        value = os.environ.get(f"_SECRET_{source_name}", "")
        if value:
            (out / target).write_text(value)
            if target != source_name:
                (out / source_name).write_text(value)

    os.chmod(secrets_dir, 0o700)


def main() -> int:
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <backend_env_file> <secrets_dir>", file=sys.stderr)
        return 1
    resolve(sys.argv[1], sys.argv[2])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
