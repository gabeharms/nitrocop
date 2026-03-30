# Autoresearch: Nitrocop violation parity with RuboCop

## Objective
Match nitrocop's offense count to RuboCop's offense count on `~/Dev/wt-gph-rspec-rip-out`, while keeping nitrocop runtime under 100ms for that workload.

## Metrics
- **Primary**: `violation_delta` (count, lower is better; target = 0)
- **Secondary**: `nitrocop_violations`, `rubocop_violations`, `nitrocop_ms`, `under_100ms`

## How to Run
`./autoresearch.sh` — emits structured `METRIC name=value` lines.

## Files in Scope
- Entire nitrocop repo (`.`), including Rust source, fixtures, config loading, formatter, and benchmark helpers.

## Off Limits
- External target repo contents (`~/Dev/wt-gph-rspec-rip-out`) are read-only for this loop.

## Constraints
- Must preserve/approach RuboCop behavior on offense counts for this workload.
- Keep nitrocop runtime under 100ms on this workload.
- Prefer behavior-compatible fixes over benchmark-only shortcuts.

## What's Been Tried
- Session initialized; baseline pending.
- Benchmark script caches RuboCop offense count in `.autoresearch_rubocop_count` to keep iterations fast while still reporting the RuboCop target.
- Next: identify largest count mismatches and close gaps with focused changes.