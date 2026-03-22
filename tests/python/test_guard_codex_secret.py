#!/usr/bin/env python3
"""Tests for guard_codex_secret.py."""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).parents[2] / "scripts" / "ci" / "guard_codex_secret.py"


def managed_auth_payload():
    return {
        "OPENAI_API_KEY": None,
        "tokens": {
            "access_token": "eyJ-access",
            "refresh_token": "rt-refresh",
            "id_token": "eyJ-id",
            "account_id": "e7-account",
        },
        "last_refresh": "2026-03-22T00:00:00Z",
    }


def run(args, payload=None):
    env = os.environ.copy()
    if payload is None:
        env.pop("CODEX_AUTH_JSON", None)
    else:
        env["CODEX_AUTH_JSON"] = json.dumps(payload)
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--from-env", "CODEX_AUTH_JSON", *args],
        capture_output=True,
        text=True,
        env=env,
    )


def test_emit_masks_outputs_commands():
    result = run(["emit-masks"], managed_auth_payload())
    assert result.returncode == 0
    assert "::add-mask::eyJ-access" in result.stdout
    assert "::add-mask::rt-refresh" in result.stdout


def test_scan_files_passes_when_clean():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".log", delete=False) as f:
        f.write("all clear")
        f.flush()
        result = run(["scan-files", f.name], managed_auth_payload())
    assert result.returncode == 0
    assert "No Codex auth secret leakage" in result.stdout


def test_scan_files_fails_on_leak():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".log", delete=False) as f:
        f.write("oops rt-refresh leaked")
        f.flush()
        result = run(["scan-files", f.name], managed_auth_payload())
    assert result.returncode != 0
    assert "potential Codex auth secret leakage" in result.stderr
    assert "refresh_token" in result.stderr


if __name__ == "__main__":
    test_emit_masks_outputs_commands()
    test_scan_files_passes_when_clean()
    test_scan_files_fails_on_leak()
    print("All tests passed.")
