#!/usr/bin/env python3
"""Tests for precompute_repair_cop_check.py."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[3] / "scripts" / "workflows"))
import precompute_repair_cop_check


def test_render_packet_includes_changed_cop_results():
    packet = precompute_repair_cop_check.render_packet(
        [
            {
                "cop": "Style/MixinUsage",
                "command": "python3 scripts/check-cop.py Style/MixinUsage --verbose --rerun --quick --clone",
                "status": 1,
                "output": "FAIL: FN increased from 0 to 38",
            }
        ]
    )
    assert "## Local Cop-Check Diagnosis" in packet
    assert "`Style/MixinUsage`" in packet
    assert "Exit status: `1`" in packet
    assert "FN increased from 0 to 38" in packet


def test_render_packet_handles_no_changed_cops():
    packet = precompute_repair_cop_check.render_packet([])
    assert "No changed cops were detected" in packet


def test_tail_lines_truncates_to_suffix():
    text = "\n".join(f"line {idx}" for idx in range(300))
    trimmed = precompute_repair_cop_check.tail_lines(text, max_lines=3)
    assert "showing last 3 of 300 lines" in trimmed
    assert "line 297" in trimmed
    assert "line 299" in trimmed
    assert "line 0" not in trimmed


if __name__ == "__main__":
    test_render_packet_includes_changed_cop_results()
    test_render_packet_handles_no_changed_cops()
    test_tail_lines_truncates_to_suffix()
    print("All tests passed.")
