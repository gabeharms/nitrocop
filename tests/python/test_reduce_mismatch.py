#!/usr/bin/env python3
"""Tests for reduce-mismatch.py."""

import importlib.util
import json
import subprocess
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

SCRIPT = Path(__file__).parents[2] / "scripts" / "reduce-mismatch.py"

spec = importlib.util.spec_from_file_location("reduce_mismatch", SCRIPT)
reduce_mismatch = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(reduce_mismatch)


def setup_function(_function=None):
    reduce_mismatch._predicate_calls = 0
    reduce_mismatch._predicate_cache.clear()


def test_rubocop_runner_uses_server_when_available():
    """Probe once, then pass --server on real RuboCop runs."""
    calls = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        if "--start-server" in cmd:
            return subprocess.CompletedProcess(cmd, 0, "", "")
        payload = {
            "files": [{
                "offenses": [
                    {"cop_name": "Style/Test", "location": {"line": 7}},
                    {"cop_name": "Other/Cop", "location": {"line": 9}},
                ]
            }]
        }
        return subprocess.CompletedProcess(cmd, 1, json.dumps(payload), "")

    with patch.object(reduce_mismatch.subprocess, "run", side_effect=fake_run):
        runner = reduce_mismatch.RubocopRunner()
        lines = runner.run("Style/Test", "/tmp/example.rb")

    assert lines == {7}
    assert calls[0] == ["bundle", "exec", "rubocop", "--start-server"]
    assert calls[1][:4] == ["bundle", "exec", "rubocop", "--server"]


def test_rubocop_runner_falls_back_without_server():
    """If the probe fails, keep using the plain RuboCop CLI."""
    calls = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        if "--start-server" in cmd:
            return subprocess.CompletedProcess(cmd, 2, "", "unsupported")
        payload = {"files": []}
        return subprocess.CompletedProcess(cmd, 0, json.dumps(payload), "")

    with patch.object(reduce_mismatch.subprocess, "run", side_effect=fake_run):
        runner = reduce_mismatch.RubocopRunner()
        lines = runner.run("Style/Test", "/tmp/example.rb")

    assert lines == set()
    assert calls[1][:3] == ["bundle", "exec", "rubocop"]
    assert "--server" not in calls[1]


def test_is_interesting_caches_repeated_candidates():
    """Repeated candidate text should not rerun the expensive predicate."""
    runner = SimpleNamespace(run=Mock(return_value=set()))

    with patch.object(reduce_mismatch, "run_nitrocop", return_value={3}) as nitrocop:
        with patch.object(reduce_mismatch, "is_parseable", return_value=True) as parseable:
            first = reduce_mismatch.is_interesting(
                "Style/Test",
                "/tmp/example.rb",
                "fp",
                runner,
                skip_rubocop=True,
                candidate_text="value\n",
            )
            second = reduce_mismatch.is_interesting(
                "Style/Test",
                "/tmp/example.rb",
                "fp",
                runner,
                skip_rubocop=True,
                candidate_text="value\n",
            )

    assert first is True
    assert second is True
    assert reduce_mismatch._predicate_calls == 1
    nitrocop.assert_called_once()
    parseable.assert_called_once()
    runner.run.assert_not_called()


def test_fp_short_circuits_before_parse_when_nitrocop_is_silent():
    """FP candidates rejected by nitrocop should not pay the parseability check."""
    runner = SimpleNamespace(run=Mock(return_value=set()))

    with patch.object(reduce_mismatch, "run_nitrocop", return_value=set()):
        with patch.object(
            reduce_mismatch,
            "is_parseable",
            side_effect=AssertionError("parseability should not be checked"),
        ):
            interesting = reduce_mismatch.is_interesting(
                "Style/Test",
                "/tmp/example.rb",
                "fp",
                runner,
                skip_rubocop=True,
                candidate_text="value\n",
            )

    assert interesting is False
    runner.run.assert_not_called()


def test_fn_short_circuits_before_parse_when_rubocop_is_silent():
    """FN candidates rejected by RuboCop should not pay the parseability check."""
    runner = SimpleNamespace(run=Mock(return_value=set()))

    with patch.object(reduce_mismatch, "run_nitrocop", return_value=set()):
        with patch.object(
            reduce_mismatch,
            "is_parseable",
            side_effect=AssertionError("parseability should not be checked"),
        ):
            interesting = reduce_mismatch.is_interesting(
                "Style/Test",
                "/tmp/example.rb",
                "fn",
                runner,
                candidate_text="value\n",
            )

    assert interesting is False
    runner.run.assert_called_once_with("Style/Test", "/tmp/example.rb")
