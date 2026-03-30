#!/bin/bash
set -euo pipefail

TARGET_REPO="${TARGET_REPO:-$HOME/Dev/wt-gph-rspec-rip-out}"
NITROCOP_BIN="${NITROCOP_BIN:-target/release/nitrocop}"
RUBOCOP_CACHE_FILE="${RUBOCOP_CACHE_FILE:-.autoresearch_rubocop_count}"

if [[ ! -d "$TARGET_REPO" ]]; then
  echo "error: target repo not found: $TARGET_REPO" >&2
  exit 2
fi

if [[ ! -x "$NITROCOP_BIN" ]]; then
  echo "error: nitrocop binary missing or not executable: $NITROCOP_BIN" >&2
  echo "hint: cargo build --release" >&2
  exit 2
fi

if [[ -f "$RUBOCOP_CACHE_FILE" ]]; then
  rubocop_violations="$(<"$RUBOCOP_CACHE_FILE")"
else
  rubocop_json="$((cd "$TARGET_REPO" && rubocop --format json) 2>/dev/null || true)"
  rubocop_violations="$(printf '%s' "$rubocop_json" | ruby -rjson -e 'raw = STDIN.read; begin; data = JSON.parse(raw); puts data.fetch("summary", {}).fetch("offense_count", 0); rescue; puts 0; end')"
  printf '%s\n' "$rubocop_violations" > "$RUBOCOP_CACHE_FILE"
fi

nitro_tmp="$(mktemp)"
start_ns="$(ruby -e 'print Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)')"
"$NITROCOP_BIN" "$TARGET_REPO" >"$nitro_tmp" 2>&1 || true
end_ns="$(ruby -e 'print Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)')"

nitrocop_violations="$(ruby -e 'raw = STDIN.read; m = raw.scan(/(\d+)\s+offenses?\s+detected/).last; puts((m && m[0]) || 0)' <"$nitro_tmp")"
rm -f "$nitro_tmp"

nitrocop_ms="$(ruby -e 's = ARGV[0].to_i; e = ARGV[1].to_i; ms = (e - s) / 1_000_000.0; printf("%.3f", ms)' "$start_ns" "$end_ns")"

if (( nitrocop_violations >= rubocop_violations )); then
  violation_delta=$((nitrocop_violations - rubocop_violations))
else
  violation_delta=$((rubocop_violations - nitrocop_violations))
fi

under_100ms="$(ruby -e 'ms = ARGV[0].to_f; puts(ms < 100.0 ? 1 : 0)' "$nitrocop_ms")"

echo "METRIC violation_delta=$violation_delta"
echo "METRIC nitrocop_violations=$nitrocop_violations"
echo "METRIC rubocop_violations=$rubocop_violations"
echo "METRIC nitrocop_ms=$nitrocop_ms"
echo "METRIC under_100ms=$under_100ms"
