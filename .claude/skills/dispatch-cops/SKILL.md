---
name: dispatch-cops
description: Dispatch cop-fix tasks to Codex agents (via GHA) for parallel corpus conformance fixes
allowed-tools: Bash(*), Read, Grep, Glob, AskUserQuestion
---

# Dispatch Cops — Remote Agent Orchestration

Dispatch cop-fix tasks to Codex agents (running in GitHub Actions) to fix
corpus conformance gaps in parallel. Each cop gets its own GHA job where
Codex edits the code, validates with cargo test, and opens a PR.

See `docs/agent-dispatch.md` for full setup instructions and architecture.

## Prerequisites

Before dispatching, verify the pipeline is set up:

```bash
# Verify the workflows exist
ls .github/workflows/agent-cop-fix.yml .github/workflows/agent-cop-check.yml
```

The user needs `CODEX_AUTH_JSON` configured in GitHub repo secrets.
See `docs/agent-dispatch.md` for setup instructions.

## Phases

### Phase 1: Triage

Find cops with real code bugs (not just config noise):

```bash
python3 scripts/agent/rank_dispatchable_cops.py
```

This runs pre-diagnostic on every cop's FP/FN examples to classify them as
code bugs (agent can fix) vs config/context issues (agent can't). Only shows
cops with at least 1 real code bug.

For MiniMax, filter to cops with 3-10 total FP+FN and mostly code bugs:

```bash
python3 scripts/agent/rank_dispatchable_cops.py --min-bugs 2 --max-total 10
```

For harder cops or overview by tier:

```bash
python3 scripts/agent/tier_cops.py --extended --tier 1   # simple FP+FN count view
python3 scripts/investigate-cop.py Department/CopName --extended --context  # deep dive
```

**Skip cops with 0 code bugs** — they're all config issues and the workflow
will auto-skip them anyway (pre-diagnostic gate).

### Phase 2: Pilot (first run only)

If this is the first time dispatching, run a 10-cop pilot:

```bash
for cop in \
  "Layout/ConditionPosition" \
  "Layout/SpaceInsideRangeLiteral" \
  "Layout/SpaceBeforeBrackets" \
  "Lint/DuplicateRegexpCharacterClassElement" \
  "Lint/ElseLayout" \
  "Lint/RescueException" \
  "Performance/ChainArrayAllocation" \
  "Style/NegatedWhile" \
  "Style/KeywordParametersOrder" \
  "Style/VariableInterpolation"; do
  gh workflow run agent-cop-fix.yml -f cop="$cop"
  sleep 5
done
```

Wait ~15-30 min, then check results:

```bash
gh pr list --search "Fix in:title" --state open
```

For each PR, check: Did CI pass? Did the agent follow TDD? Did it stay within
its cop's files? Ask the user if the results look good before scaling.

### Phase 3: Batch Dispatch

Ask the user which tier to dispatch. Then dispatch:

```bash
# Dispatch all cops with real code bugs (minimax, default)
python3 scripts/agent/rank_dispatchable_cops.py --json | \
  jq -r '.[].cop' | while read cop; do
  gh workflow run agent-cop-fix.yml -f cop="$cop"
  sleep 5
done

# Or dispatch a specific tier with Codex for harder cops
python3 scripts/agent/tier_cops.py --extended --tier 2 --names | while read cop; do
  gh workflow run agent-cop-fix.yml -f cop="$cop" -f backend="codex"
  sleep 5
done
```

Monitor progress:

```bash
# Open PRs
gh pr list --state open --limit 50

# PRs with passing CI
gh pr list --state open --search "status:success" --limit 50
```

### Phase 4: Review + Merge

Help the user review and merge PRs. For each PR:

```bash
gh pr view <number>
gh pr checks <number>
gh pr diff <number>
```

If CI passes and the diff looks right:

```bash
gh pr merge <number> --squash
```

### Phase 5: Retry Failures

Find cops with failed PRs:

```bash
gh pr list --state open --search "status:failure" --limit 50
```

Retry each with stronger model:

```bash
gh workflow run agent-cop-fix.yml -f cop="Department/CopName" -f mode=retry
```

For specific issues, add context:

```bash
gh workflow run agent-cop-fix.yml \
  -f cop="Department/CopName" \
  -f mode=retry \
  -f extra_context="<what went wrong>"
```

### Phase 6: Validate

After merging a batch (~20-50 PRs), run the full corpus oracle:

```bash
gh workflow run corpus-oracle.yml -f corpus_size=extended
```

Wait ~90 min, then check results:

```bash
python3 scripts/agent/tier_cops.py --extended
```

## Arguments

- `/dispatch-cops` — start from Phase 1 (triage)
- `/dispatch-cops pilot` — jump to Phase 2 (10-cop pilot)
- `/dispatch-cops tier1` — jump to Phase 3 (batch dispatch Tier 1)
- `/dispatch-cops tier2` — jump to Phase 3 (batch dispatch Tier 2)
- `/dispatch-cops retry` — jump to Phase 5 (retry failures)
- `/dispatch-cops status` — show current PR status and merge candidates
- `/dispatch-cops validate` — jump to Phase 6 (trigger corpus oracle)

## Important Notes

- Codex runs inside GHA — full Rust build environment with cache
- The task prompt contains all context the agent needs
- `workflow_dispatch` requires write access — safe on public repos
- Retries auto-close stale PRs and include prior attempt context
- Monitor ChatGPT usage at chatgpt.com/codex/settings/usage — dispatch in small batches
