# land-agent-prs

Review and merge PRs created by the agent cop fix workflow.

## Trigger

User runs `/land-agent-prs` (optionally with filters like `--dry-run`, `--cop Style/*`).

## Workflow

### 1. List open agent PRs

```bash
gh pr list --repo 6/nitrocop --label agent-fix --state open \
  --json number,title,headRefName,statusCheckRollup,labels,createdAt \
  --jq '.[] | "\(.number)\t\(.title)\t\(.labels | map(.name) | join(","))"'
```

### 2. Categorize each PR

For each open PR, check:
- **Validation status**: look for `validation-failed` label or validation comment
- **CI status**: `gh pr checks <number>` (may be empty if checks weren't dispatched)
- **Mergeable**: no conflicts with main

Categorize into:
- **Ready**: validation passed, no `validation-failed` label, tests OK
- **Failed**: has `validation-failed` label or failing checks
- **Fixable**: CI passes but code has quality nits worth fixing before merge
- **Pending**: no validation results yet

### 2b. Review code quality and fix nits

For each PR that passes CI, review the diff for quality issues common in agent-generated code:
- Missing explicit parentheses for operator precedence clarity
- Overly verbose or unclear comments
- Unnecessary complexity that could be simplified

If nits are found, check out the PR branch, fix them, commit, and push before merging.

### 3. Present summary table

Show a table like:
```
| PR  | Cop                        | Status  | Action     |
|-----|----------------------------|---------|------------|
| #102| Style/VariableInterpolation| Ready   | Will merge |
| #105| Lint/EmptyBlock            | Failed  | Skip       |
| #108| Style/SymbolProc           | Pending | Skip       |
```

### 4. Merge ready PRs

For each "Ready" PR:
```bash
gh pr merge <number> --squash --delete-branch
```

If `--dry-run` was specified, show what would be merged but don't merge.

### 5. Report

Summarize: N merged, N failed (skipped), N pending (skipped).

## Rules

- Only merge PRs with the `agent-fix` label
- Never merge PRs with the `validation-failed` label
- Squash merge to keep main history clean
- Delete the branch after merge
- If unsure about a PR, skip it and note why
- Don't merge PRs that have merge conflicts — note them for manual resolution
