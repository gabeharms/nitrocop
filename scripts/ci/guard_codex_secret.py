#!/usr/bin/env python3
"""Register masks and scan files for Codex auth secret leakage."""

import argparse
import glob
import json
import os
import sys


def _nonempty_string(value) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _load_secret(var_name: str):
    raw = os.environ.get(var_name, "")
    if not raw.strip():
        raise ValueError(f"{var_name} is missing or empty")

    parsed = None
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError:
        parsed = None
    return raw, parsed


def _collect_values(raw_secret: str, parsed) -> list[tuple[str, str]]:
    values = [("raw_auth_json", raw_secret)]

    if isinstance(parsed, dict):
        api_key = parsed.get("OPENAI_API_KEY")
        if _nonempty_string(api_key):
            values.append(("openai_api_key", api_key))

        tokens = parsed.get("tokens")
        if isinstance(tokens, dict):
            for key in ("access_token", "refresh_token", "id_token", "account_id"):
                value = tokens.get(key)
                if _nonempty_string(value):
                    values.append((key, value))

    deduped = []
    seen = set()
    for label, value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append((label, value))
    return deduped


def _expand_patterns(patterns: list[str]) -> list[str]:
    expanded = []
    seen = set()
    for pattern in patterns:
        matches = glob.glob(os.path.expanduser(pattern), recursive=True)
        if matches:
            candidates = matches
        else:
            candidates = [os.path.expanduser(pattern)]
        for candidate in candidates:
            if candidate in seen:
                continue
            seen.add(candidate)
            expanded.append(candidate)
    return expanded


def emit_masks(var_name: str) -> int:
    raw_secret, parsed = _load_secret(var_name)
    for _, value in _collect_values(raw_secret, parsed):
        print(f"::add-mask::{value}")
    return 0


def scan_files(var_name: str, patterns: list[str]) -> int:
    raw_secret, parsed = _load_secret(var_name)
    secret_values = _collect_values(raw_secret, parsed)
    leaked = []

    for path in _expand_patterns(patterns):
        if not os.path.isfile(path):
            continue
        try:
            with open(path, "r", errors="ignore") as f:
                content = f.read()
        except OSError:
            continue

        for label, value in secret_values:
            if value and value in content:
                leaked.append((path, label))
                break

    if leaked:
        print("ERROR: potential Codex auth secret leakage detected in generated files:", file=sys.stderr)
        for path, label in leaked:
            print(f"  - {path} ({label})", file=sys.stderr)
        return 1

    print("No Codex auth secret leakage detected in generated files.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--from-env",
        default="CODEX_AUTH_JSON",
        help="Environment variable holding the auth JSON (default: CODEX_AUTH_JSON)",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("emit-masks")

    scan = subparsers.add_parser("scan-files")
    scan.add_argument("patterns", nargs="+", help="File paths or glob patterns to scan")

    args = parser.parse_args()

    try:
        if args.command == "emit-masks":
            return emit_masks(args.from_env)
        if args.command == "scan-files":
            return scan_files(args.from_env, args.patterns)
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    return 1


if __name__ == "__main__":
    raise SystemExit(main())
