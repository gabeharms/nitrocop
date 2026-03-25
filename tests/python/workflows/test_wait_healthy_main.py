#!/usr/bin/env python3
"""Tests for scripts/workflows/wait_healthy_main.py."""

from __future__ import annotations

import sys
from pathlib import Path
from unittest.mock import patch

SCRIPT_DIR = Path(__file__).parents[3] / "scripts" / "workflows"
sys.path.insert(0, str(SCRIPT_DIR))

import wait_healthy_main


def _mock_run(head_sha: str, checks_run: dict | None):
    """Return patchers for get_head_sha and get_latest_checks_run."""
    return (
        patch.object(wait_healthy_main, "get_head_sha", return_value=head_sha),
        patch.object(wait_healthy_main, "get_latest_checks_run", return_value=checks_run),
        patch.object(wait_healthy_main.time, "sleep"),
    )


def test_success_same_sha(capsys):
    """Green checks on HEAD — proceed immediately."""
    sha = "abc1234567890"
    p1, p2, p3 = _mock_run(sha, {"headSha": sha, "conclusion": "success", "status": "completed"})
    with p1, p2, p3:
        wait_healthy_main.main.__wrapped__ = None  # reset argparse
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "5", "--interval", "1"]):
            wait_healthy_main.main()
    out = capsys.readouterr().out
    assert "green" in out.lower()


def test_success_different_sha(capsys):
    """Green checks on older SHA (HEAD is [skip ci]) — proceed."""
    p1, p2, p3 = _mock_run("head111", {"headSha": "old222", "conclusion": "success", "status": "completed"})
    with p1, p2, p3:
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "5", "--interval", "1"]):
            wait_healthy_main.main()
    out = capsys.readouterr().out
    assert "green" in out.lower()


def test_no_runs(capsys):
    """No checks.yml runs at all — proceed."""
    p1, p2, p3 = _mock_run("head111", None)
    with p1, p2, p3:
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "5", "--interval", "1"]):
            wait_healthy_main.main()
    out = capsys.readouterr().out
    assert "No checks" in out


def test_skip_ci_failed_old_run(capsys):
    """HEAD is [skip ci], latest checks (old SHA) failed — proceed with warning."""
    p1, p2, p3 = _mock_run("head111", {"headSha": "old222", "conclusion": "failure", "status": "completed"})
    with p1, p2, p3:
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "5", "--interval", "1"]):
            wait_healthy_main.main()
    out = capsys.readouterr().out
    assert "no checks" in out.lower()


def test_in_progress_waits_then_succeeds(capsys):
    """Checks in_progress on HEAD — wait, then succeed."""
    sha = "head111"
    call_count = 0
    def mock_checks(repo):
        nonlocal call_count
        call_count += 1
        if call_count < 3:
            return {"headSha": sha, "conclusion": None, "status": "in_progress"}
        return {"headSha": sha, "conclusion": "success", "status": "completed"}

    with (
        patch.object(wait_healthy_main, "get_head_sha", return_value=sha),
        patch.object(wait_healthy_main, "get_latest_checks_run", side_effect=mock_checks),
        patch.object(wait_healthy_main.time, "sleep"),
    ):
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "300", "--interval", "1"]):
            wait_healthy_main.main()
    assert call_count == 3


def test_timeout_exits_nonzero():
    """Checks never go green — exit 1 after max-wait."""
    sha = "head111"
    p1, p2, p3 = _mock_run(sha, {"headSha": sha, "conclusion": None, "status": "in_progress"})
    with p1, p2, p3:
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "2", "--interval", "1"]):
            try:
                wait_healthy_main.main()
                assert False, "Should have called sys.exit(1)"
            except SystemExit as e:
                assert e.code == 1


def test_skip_ci_in_progress_old_run_waits(capsys):
    """HEAD is [skip ci], old checks still in_progress — should wait."""
    call_count = 0
    def mock_checks(repo):
        nonlocal call_count
        call_count += 1
        if call_count < 3:
            return {"headSha": "old222", "conclusion": None, "status": "in_progress"}
        return {"headSha": "old222", "conclusion": "success", "status": "completed"}

    with (
        patch.object(wait_healthy_main, "get_head_sha", return_value="head111"),
        patch.object(wait_healthy_main, "get_latest_checks_run", side_effect=mock_checks),
        patch.object(wait_healthy_main.time, "sleep"),
    ):
        with patch("sys.argv", ["prog", "--repo", "test/repo", "--max-wait", "300", "--interval", "1"]):
            wait_healthy_main.main()
    assert call_count == 3
