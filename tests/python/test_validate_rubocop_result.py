#!/usr/bin/env python3
"""Tests for bench/corpus/validate_rubocop_result.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).parents[2] / "bench" / "corpus" / "validate_rubocop_result.py"


def run_validator(result_json: dict, repo_dir: str) -> subprocess.CompletedProcess:
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
        json.dump(result_json, f)
        f.flush()
        return subprocess.run(
            [sys.executable, str(SCRIPT), f.name, repo_dir],
            capture_output=True, text=True,
        )


def test_valid_result():
    result = run_validator(
        {"files": [
            {"path": "/tmp/repos/my_repo/app/models/user.rb", "offenses": []},
            {"path": "/tmp/repos/my_repo/lib/tasks/seed.rb", "offenses": []},
        ]},
        "/tmp/repos/my_repo",
    )
    assert result.returncode == 0


def test_valid_empty_files():
    result = run_validator({"files": []}, "/tmp/repos/my_repo")
    assert result.returncode == 0


def test_valid_no_files_key():
    result = run_validator({"summary": {"offense_count": 0}}, "/tmp/repos/my_repo")
    assert result.returncode == 0


def test_poisoned_cross_repo_path():
    result = run_validator(
        {"files": [
            {"path": "/tmp/repos/my_repo/app/models/user.rb", "offenses": []},
            {"path": "/tmp/repos/OTHER_REPO/lib/foo.rb", "offenses": []},
        ]},
        "/tmp/repos/my_repo",
    )
    assert result.returncode == 1
    assert "POISONED" in result.stderr


def test_poisoned_workspace_root():
    result = run_validator(
        {"files": [
            {"path": "/home/runner/work/nitrocop/nitrocop/Gemfile", "offenses": []},
        ]},
        "/home/runner/work/nitrocop/nitrocop/repos/some_repo",
    )
    assert result.returncode == 1
    assert "POISONED" in result.stderr


def test_trailing_slash_normalization():
    """repo_dir with trailing slash should still work."""
    result = run_validator(
        {"files": [
            {"path": "/tmp/repos/my_repo/foo.rb", "offenses": []},
        ]},
        "/tmp/repos/my_repo/",
    )
    assert result.returncode == 0


def test_missing_args():
    result = subprocess.run(
        [sys.executable, str(SCRIPT)],
        capture_output=True, text=True,
    )
    assert result.returncode == 2
