#!/usr/bin/env bash
set -euo pipefail

TARGET_REPO="$HOME/Dev/wt-gph-rspec-rip-out"
NITROCOP_BIN="target/release/nitrocop"
RUBOCOP_BASELINE=3153
RUNS=7

# Fast pre-checks
[[ -x "$NITROCOP_BIN" ]] || { echo "error: missing executable $NITROCOP_BIN" >&2; exit 2; }
[[ -d "$TARGET_REPO" ]] || { echo "error: missing target repo $TARGET_REPO" >&2; exit 2; }

runtimes=()
violations=()

for i in $(seq 1 "$RUNS"); do
  start_ms=$(python3 -c 'import time; print(int(time.time()*1000))')
  set +e
  output="$($NITROCOP_BIN "$TARGET_REPO" 2>&1)"
  status=$?
  set -e
  end_ms=$(python3 -c 'import time; print(int(time.time()*1000))')

  runtime_ms=$(( end_ms - start_ms ))

  # Treat normal lint exit code (1 = offenses found) as successful benchmark execution.
  if [[ "$status" -ne 0 && "$status" -ne 1 ]]; then
    echo "error: nitrocop exited with status $status on run $i" >&2
    exit "$status"
  fi

  summary_line=$(printf '%s\n' "$output" | grep -E '[0-9]+ files inspected, [0-9]+ offenses detected' | tail -n1 || true)
  if [[ -z "$summary_line" ]]; then
    echo "error: could not parse nitrocop offense summary on run $i" >&2
    exit 2
  fi

  nitrocop_violations=$(printf '%s\n' "$summary_line" | sed -E 's/.* inspected, ([0-9]+) offenses detected.*/\1/')

  runtimes+=("$runtime_ms")
  violations+=("$nitrocop_violations")

done

# Use median runtime for stability.
sorted_runtimes=$(printf '%s\n' "${runtimes[@]}" | sort -n)
median_runtime_ms=$(printf '%s\n' "$sorted_runtimes" | awk 'NR==4')

# Offense count should be stable run-to-run; use last value and expose drift signal.
last_idx=$(( ${#violations[@]} - 1 ))
nitrocop_violations="${violations[$last_idx]}"
min_violations=$(printf '%s\n' "${violations[@]}" | sort -n | head -n1)
max_violations=$(printf '%s\n' "${violations[@]}" | sort -n | tail -n1)
violation_spread=$(( max_violations - min_violations ))

violation_gap=$(( nitrocop_violations > RUBOCOP_BASELINE ? nitrocop_violations - RUBOCOP_BASELINE : RUBOCOP_BASELINE - nitrocop_violations ))
runtime_budget_hit=$(( median_runtime_ms <= 150 ? 1 : 0 ))

# Primary optimization target now: runtime, with parity treated as a hard constraint.
if [[ "$violation_gap" -ne 0 ]]; then
  echo "error: parity regression detected (nitrocop=$nitrocop_violations, rubocop=$RUBOCOP_BASELINE)" >&2
  exit 2
fi

printf 'METRIC nitrocop_ms=%s\n' "$median_runtime_ms"
printf 'METRIC violation_gap=%s\n' "$violation_gap"
printf 'METRIC nitrocop_violations=%s\n' "$nitrocop_violations"
printf 'METRIC runtime_budget_hit=%s\n' "$runtime_budget_hit"
printf 'METRIC violation_spread=%s\n' "$violation_spread"
