#!/usr/bin/env python3
"""Tests for render_corpus_oracle_pr.py."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[3] / "scripts" / "workflows"))
import render_corpus_oracle_pr as renderer


def test_build_metadata_for_full_refresh():
    metadata = renderer.build_metadata(
        repo_filter="all",
        run_number="147",
        run_url="https://github.com/6/nitrocop/actions/runs/123",
        changed_files=[
            "README.md",
            "docs/corpus.md",
            "src/resources/tiers.json",
            "docs/cop_coverage.md",
        ],
    )

    expected = "[skip ci] Corpus oracle: refresh tiers.json, README, corpus report, and cop coverage report"
    assert metadata["commit_message"] == expected
    assert metadata["pr_title"] == expected
    assert "Automated corpus oracle refresh from [run #147]" in metadata["pr_body"]
    assert "| **Scope** | `all repos` |" in metadata["pr_body"]
    assert "| **CI** | intentionally skipped via `[skip ci]` |" in metadata["pr_body"]
    assert "- `src/resources/tiers.json`: Stable and preview tiers regenerated" in metadata["pr_body"]
    assert "- `README.md`: Top-level conformance summary refreshed" in metadata["pr_body"]
    assert "- `docs/corpus.md`: Full corpus report regenerated." in metadata["pr_body"]
    assert "- `docs/cop_coverage.md`: Cop coverage summary refreshed" in metadata["pr_body"]


if __name__ == "__main__":
    test_build_metadata_for_full_refresh()
    print("All tests passed.")
