#!/usr/bin/env python3
"""Rank cops by dispatchability: real code bugs vs config noise.

Runs pre-diagnostic on each cop's FP/FN examples to classify them as
code bugs (agent can fix) vs config/context issues (agent can't fix).
Only shows cops with at least 1 real code bug.

Usage:
    python3 scripts/agent/rank_dispatchable_cops.py
    python3 scripts/agent/rank_dispatchable_cops.py --min-bugs 3 --max-total 10
    python3 scripts/agent/rank_dispatchable_cops.py --binary target/debug/nitrocop
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from shared.corpus_download import download_corpus_results


def run_nitrocop(binary: Path, cwd: str, cop: str) -> list[dict]:
    proc = subprocess.run(
        [str(binary), "--force-default-config", "--only", cop,
         "--format", "json", "test.rb"],
        capture_output=True, text=True, timeout=15, cwd=cwd,
    )
    if proc.stdout.strip():
        try:
            return json.loads(proc.stdout).get("offenses", [])
        except json.JSONDecodeError:
            pass
    return []


def extract_lines(src: list[str]) -> tuple[list[str], str | None]:
    lines, offense = [], None
    for s in src:
        is_off = s.strip().startswith(">>>")
        cleaned = re.sub(r"^(>>>\s*)?\s*\d+:\s?", "", s)
        lines.append(cleaned)
        if is_off:
            offense = cleaned
    return lines, offense


def diagnose_cop(binary: Path, cop: str, examples: list, kind: str) -> tuple[int, int]:
    """Returns (code_bugs, config_issues) for a list of examples."""
    bugs, cfg = 0, 0
    for ex in examples[:5]:
        if not isinstance(ex, dict) or not ex.get("src"):
            continue
        lines, offense = extract_lines(ex["src"])
        if not lines:
            continue
        tmp = tempfile.mkdtemp()
        try:
            with open(os.path.join(tmp, "test.rb"), "w") as f:
                f.write("\n".join(lines) + "\n")
            offenses = run_nitrocop(binary, tmp, cop)
            if not offenses and offense:
                with open(os.path.join(tmp, "test.rb"), "w") as f:
                    f.write(offense + "\n")
                offenses = run_nitrocop(binary, tmp, cop)
            detected = len(offenses) > 0
            if (kind == "fn" and not detected) or (kind == "fp" and detected):
                bugs += 1
            else:
                cfg += 1
        except Exception:
            pass
        finally:
            try:
                os.unlink(os.path.join(tmp, "test.rb"))
                os.rmdir(tmp)
            except OSError:
                pass
    return bugs, cfg


def main():
    parser = argparse.ArgumentParser(description="Rank cops by dispatchability")
    parser.add_argument("--binary", type=Path,
                        help="Path to nitrocop binary (default: auto-detect)")
    parser.add_argument("--min-bugs", type=int, default=1,
                        help="Minimum code bugs to show (default: 1)")
    parser.add_argument("--max-total", type=int, default=15,
                        help="Maximum FP+FN total (default: 15)")
    parser.add_argument("--min-total", type=int, default=3,
                        help="Minimum FP+FN total (default: 3)")
    parser.add_argument("--min-matches", type=int, default=50,
                        help="Minimum corpus matches (default: 50)")
    parser.add_argument("--json", action="store_true",
                        help="Output JSON instead of table")
    args = parser.parse_args()

    # Find binary
    binary = args.binary
    if not binary:
        for candidate in [
            Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "debug" / "nitrocop",
            Path("target-linux/debug/nitrocop"),
            Path("target/debug/nitrocop"),
        ]:
            if candidate.exists():
                binary = candidate.resolve()
                break
    if not binary or not binary.exists():
        print("Error: nitrocop binary not found. Build with 'cargo build' or pass --binary",
              file=sys.stderr)
        sys.exit(1)

    print(f"Using binary: {binary}", file=sys.stderr)

    p, _, _ = download_corpus_results(prefer="extended")
    data = json.loads(p.read_text())

    results = []
    for e in sorted(data["by_cop"], key=lambda x: x.get("fp", 0) + x.get("fn", 0)):
        fp, fn = e.get("fp", 0), e.get("fn", 0)
        total = fp + fn
        if total < args.min_total or total > args.max_total:
            continue
        if e.get("matches", 0) < args.min_matches:
            continue

        cop = e["cop"]
        fn_bugs, fn_cfg = diagnose_cop(binary, cop, e.get("fn_examples", []), "fn")
        fp_bugs, fp_cfg = diagnose_cop(binary, cop, e.get("fp_examples", []), "fp")
        bugs = fn_bugs + fp_bugs
        cfg = fn_cfg + fp_cfg

        if bugs >= args.min_bugs:
            results.append({
                "cop": cop, "fp": fp, "fn": fn,
                "code_bugs": bugs, "config_issues": cfg,
                "matches": e.get("matches", 0),
            })

    if args.json:
        json.dump(results, sys.stdout, indent=2)
    else:
        print(f"\n{'Cop':<42} {'FP':>3} {'FN':>3} {'Bugs':>4} {'Cfg':>4} {'Matches':>7}")
        print("-" * 68)
        for r in results:
            print(f"{r['cop']:<42} {r['fp']:>3} {r['fn']:>3} {r['code_bugs']:>4} {r['config_issues']:>4} {r['matches']:>7}")
        print(f"\n{len(results)} cops with {args.min_bugs}+ code bugs", file=sys.stderr)


if __name__ == "__main__":
    main()
