# Autoresearch: Nitrocop violation-count parity with RuboCop

## Objective
Make `nitrocop` report the same offense count as RuboCop for `~/Dev/wt-gph-rspec-rip-out`.

RuboCop baseline (fixed target) is measured once and treated as constant for this session.

## Metrics
- **Primary**: `nitrocop_ms` (ms, lower is better) — median runtime across 7 runs in `autoresearch.sh`.
- **Secondary**:
  - `violation_gap` (count) — absolute difference from RuboCop baseline (`3153`), must remain `0`.
  - `nitrocop_violations` (count) — raw nitrocop offense count.
  - `runtime_budget_hit` (0/1) — `1` if `nitrocop_ms <= 150`, else `0`.
  - `violation_spread` (count) — max-min offense count across the 7 repeated runs.

## How to Run
`./autoresearch.sh` — emits structured `METRIC name=value` lines.

## Files in Scope
- Entire nitrocop repository (`.`) is in scope per user instruction.
- `autoresearch.md` — session context and running notes.
- `autoresearch.sh` — benchmark harness and metric extraction.
- `autoresearch.jsonl` — experiment log.
- `autoresearch.ideas.md` — deferred promising ideas.

## Off Limits
- Do not modify `~/Dev/wt-gph-rspec-rip-out` (workload repo used for measurement).
- Do not re-run RuboCop baseline repeatedly; use fixed baseline value.

## Constraints
- RuboCop baseline offense count is fixed at `3153`.
- Hard constraint: keep nitrocop offense count equal to baseline (gap must stay `0`); `autoresearch.sh` exits non-zero on parity regressions.
- Keep nitrocop runtime under `150ms` (this is now the optimized metric once parity is achieved).

## What's Been Tried
- Initial measurement:
  - RuboCop count confirmed once: `3153` offenses.
  - nitrocop currently reports `3153` offenses on target workload (gap `0`).
- Harness reliability fix:
  - First benchmark attempt crashed because macOS `date` does not support `%N` nanoseconds.
  - Switched timing to portable Python millisecond timestamps.
- Current status:
  - Parity target is already met (`violation_gap=0`).
  - Runtime sample was `46ms`, below the `150ms` budget.
- Session pivot:
  - Since parity gap already reached its floor (`0`), primary metric switched to `nitrocop_ms` with parity enforced as a hard guardrail.
  - Benchmark updated to 7-run median to reduce noise and avoid overfitting to one-off timing jitter.
- Next focus: preserve `violation_gap=0` while reducing runtime variance or median runtime without benchmark-specific shortcuts.
