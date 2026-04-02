# nitrocop vs RuboCop Gap — Status Report

**Branch:** `autoresearch/autocorrect-gap-20260330`
**Target project:** `~/Dev/wt-gph-rspec-rip-out`
**Date:** 2026-04-02

## Goal

Minimize the per-cop offense count differences between nitrocop and RuboCop when linting the `wt-gph-rspec-rip-out` project. The project uses the `standard` gem with `AllCops: NewCops: enable` in `.rubocop.yml`.

## Starting Point

**Original numbers (before any work):**
- nitrocop: 4021 offenses (6143 files)
- RuboCop: 3990 offenses (6143 files)
- **Gap: +31** (28 cops with differences)

## Current State

**After all fixes (fresh cache, 2026-04-02):**
- nitrocop: 4021 offenses (6143 files)
- RuboCop: 3990 offenses (6143 files)
- **Gap: +31** (21 cops with differences — down from 28)

The total gap is numerically the same as the starting point because fixing FNs (where nitrocop missed offenses) increased nitrocop's count by the same amount that fixing FPs decreased it. The key metric is **per-cop accuracy**: 10 cops were fixed to exact match, reducing mismatched cops from 28 to 21.

Excluding the unfixable Lint/Syntax parser difference (+67), the effective gap improved from -47 to -36 (11 more correctly detected offenses).

### Per-Cop Breakdown (21 cops with differences)

```
Cop                                                  Nitro   Rubo    Gap
--------------------------------------------------------------------------------
Lint/Syntax                                           1523   1456    +67
Standard/BlockSingleLineBraces                           0     23    -23
Layout/IndentationWidth                                 18     24     -6
Layout/HashAlignment                                     6      9     -3
Lint/MissingCopEnableDirective                           3      0     +3
Lint/UselessAssignment                                 241    244     -3
Performance/StringBytesize                               6      3     +3
Rails/IndexBy                                           12      9     +3
Rails/IndexWith                                         12      9     +3
Rails/Pick                                               6      3     +3
Rails/SafeNavigationWithBlank                            6      3     +3
Style/EmptyLiteral                                       0      3     -3
Style/QuotedSymbols                                      0      3     -3
Style/RedundantParentheses                               6      9     -3
Style/Sample                                            15     18     -3
Style/RedundantRegexpEscape                              2      4     -2
Lint/CircularArgumentReference                           2      3     -1
Lint/RedundantTypeConversion                             3      4     -1
Lint/RedundantWithObject                                 2      3     -1
Lint/SymbolConversion                                    9     10     -1
Style/RedundantDoubleSplatHashBraces                     2      3     -1
--------------------------------------------------------------------------------
TOTAL                                                 4021   3990    +31

FP total (nitro over-reports): +85
FN total (nitro under-reports): -54
```

## Cops Fixed to Exact Match

10 cops were fixed across sessions 1-3:

| Cop | Before | After | Session |
|-----|--------|-------|---------|
| Style/RedundantBegin | nitro=3, rubo=10 (-7) | 10=10 | 2 |
| Layout/MultilineOperationIndentation | nitro=0, rubo=6 (-6) | 6=6 | 2 |
| Lint/ShadowedException | nitro=6, rubo=3 (+3) | 3=3 | 3 |
| Style/OneLineConditional | nitro=3, rubo=6 (-3) | 6=6 | 3 |
| Style/TrailingCommaInArguments | nitro=12, rubo=15 (-3) | 15=15 | 3 |
| Rails/HttpPositionalArguments | nitro=0, rubo=3 (-3) | 3=3 | 3 |
| Style/ItAssignment | nitro=1, rubo=3 (-2) | 3=3 | 3 |
| Style/ComparableClamp | nitro=0, rubo=2 (-2) | 2=2 | 3 |
| Style/SlicingWithRange | nitro=1, rubo=2 (-1) | 2=2 | 3 |
| Style/RedundantSelf | nitro=50, rubo=49 (+1) | 49=49 | 1 |

## All Fixes Applied (Unstaged)

31 files modified, 427 insertions, 61 deletions. All changes are unstaged.

### Session 1 Fixes (prior session)

#### 1. Lint/Syntax Dedup (`src/linter.rs`)
- Changed `suppress_secondary_syntax_diagnostics()` to deduplicate by `(line, column, message)` instead of just `(line, column)`.

#### 2. Style/TrailingCommaInHashLiteral (`src/cop/style/trailing_comma_in_hash_literal.rs`)
- Added `no_elements_on_same_line` check for `comma`/`consistent_comma` style.

#### 3. Style/TrailingCommaInArrayLiteral (`src/cop/style/trailing_comma_in_array_literal.rs`)
- Made `no_elements_on_same_line` function `pub(crate)` for cross-cop sharing.

#### 4. Layout/MultilineMethodCallIndentation (`src/cop/layout/multiline_method_call_indentation.rs`)
- Reset `in_hash_value` flag when entering parenthesized args.

#### 5. Layout/ArgumentAlignment (`src/cop/layout/argument_alignment.rs`)
- Added check for first element under `with_fixed_indentation` style.

#### 6. Layout/ArrayAlignment (`src/cop/layout/array_alignment.rs`)
- Added `check_first` parameter to `check_element_alignment`.

#### 7. Layout/LeadingCommentSpace (`src/cop/layout/leading_comment_space.rs`)
- Implemented `AllowRBSInlineAnnotation` config support for `#:` comments.

#### 8. Style/RedundantSelf (`src/cop/style/redundant_self.rs`)
- Added `collect_multi_target_locals()` for destructured parameters with `RequiredParameterNode` handling.

### Session 2 Fixes

#### 9. Layout/MultilineOperationIndentation (`src/cop/layout/multiline_operation_indentation.rs`)
- Removed overly-permissive escape hatches. Added `arg_col == recv_col` for "indented" style.

#### 10. Style/RedundantBegin (`src/cop/style/redundant_begin.rs`)
- Added standalone `begin...end` detection with `in_loop_body` flag for do-while loops and `body_has_rescue_modifier()` for inline rescue.

### Session 3 Fixes

#### 11. Lint/ShadowedException (`src/cop/lint/shadowed_exception.rs`)
- Removed `a == b` early return from `equivalent_exception_classes()`. Identical unknown exception names in the same rescue clause (`rescue Foo, Foo`) are `Lint/DuplicateRescueException`, not `Lint/ShadowedException`.

#### 12. Style/OneLineConditional (`src/cop/style/one_line_conditional.rs`)
- Removed `then_keyword_loc().is_none()` guard for both if and unless branches. Ruby allows semicolons as separators (`if cond; body else other end`), but Prism returns `nil` for `then_keyword_loc` in that case. The same-line check (`start_line != end_line`) already prevents FPs on multiline conditionals.

#### 13. Style/TrailingCommaInArguments (`src/cop/style/trailing_comma_in_arguments.rs`)
- Added single-line trailing comma removal to the `comma`/`consistent_comma` branch. Previously only the `no_comma` default branch handled unwanted trailing commas.

#### 14. Rails/HttpPositionalArguments (`src/cop/rails/http_positional_arguments.rs`)
- Relaxed `arg_list.len() >= 3` to `>= 2`. The 2-arg pattern `get path, {params}` (explicit HashNode as second argument) is also an offense.

#### 15. Style/ItAssignment (`src/cop/style/it_assignment.rs`)
- Added `REQUIRED_PARAMETER_NODE` and `OPTIONAL_PARAMETER_NODE` to `interested_node_types`. Method parameters named `it` (e.g., `def foo(it)`, `def bar(it = 5)`) should also be flagged.

#### 16. Style/ComparableClamp (`src/cop/style/comparable_clamp.rs`)
- Added `[[x, low].max, high].min` and `[[x, high].min, low].max` array/method-call pattern detection via new `extract_array_clamp()` function. Previously only handled the `if/elsif/else` form.

#### 17. Style/SlicingWithRange (`src/cop/style/slicing_with_range.rs`)
- Added handling for `nil` as left operand in range. `items[nil..42]` now suggests `items[..42]`. Prism parses explicit `nil` as a `NilNode`, but the cop only checked for integer left operands.

## Remaining Gap Analysis

### Unfixable

#### Lint/Syntax +67
Inherent difference between Prism (nitrocop's parser) and the Parser gem (RuboCop's parser). They produce different parse error counts/messages. Cannot be resolved without switching parsers.

### Unimplemented

#### Standard/BlockSingleLineBraces -23
A `standard` gem cop, not part of core RuboCop. Would require implementing from scratch.

### Deferred (risky tradeoff)

#### Layout/IndentationWidth -6
- 9 FNs: 6 from tab-indented files, 3 from begin-end alignment edge cases.
- 3 FPs: `DefEndAlignment` misaligned `end` edge cases.
- Tab handling would regress 159 FPs on the corpus. All differences are in corpus fixtures.

#### Layout/HashAlignment -3
- Previously attempted, reverted due to +252 FP regression on corpus.

### Corpus fixture artifacts (not real-world code)

All remaining differences are from files under `benchmark/rubocop_corpus/fixtures/` (synthetic test data):

**FP cops (nitro over-reports):**
| Cop | Gap | Root Cause |
|-----|-----|------------|
| Lint/MissingCopEnableDirective | +3 | Cross-cop config check: disabled cop doesn't need matching enable. Non-trivial to implement. |
| Performance/StringBytesize | +3 | Malformed single-line multi-expression corpus fixture |
| Rails/IndexBy | +3 | Malformed single-line multi-expression corpus fixture |
| Rails/IndexWith | +3 | Malformed single-line multi-expression corpus fixture |
| Rails/Pick | +3 | Malformed single-line multi-expression corpus fixture |
| Rails/SafeNavigationWithBlank | +3 | Malformed single-line multi-expression corpus fixture |

**FN cops (nitro under-reports):**
| Cop | Gap | Root Cause |
|-----|-----|------------|
| Style/EmptyLiteral | -3 | Config resolution / frozen_string_literal issue with standard gem |
| Style/QuotedSymbols | -3 | Config inheritance from standard gem not resolving StringLiterals style |
| Style/RedundantParentheses | -3 | Nested parentheses edge case (3 nested levels) |
| Style/Sample | -3 | `shuffle[n]` for non-zero n not detected |
| Lint/UselessAssignment | -3 | `a += e` inside block not detected as useless |
| Style/RedundantRegexpEscape | -2 | `\/` in `%r{}` and `\-` outside char class not flagged |
| Lint/CircularArgumentReference | -1 | Double-nested default (`pie = pie = pie`) |
| Lint/RedundantTypeConversion | -1 | `Set.new.to_set` not recognized |
| Lint/RedundantWithObject | -1 | `each.with_object` chain not detected |
| Lint/SymbolConversion | -1 | `:sym.to_sym` missed in cross-cop fixture |
| Style/RedundantDoubleSplatHashBraces | -1 | `**{...}.merge(other)` chain not detected |

### Investigated but skipped

#### Lint/MissingCopEnableDirective +3
Initial investigation assumed `MaximumRangeSize: .inf` was the root cause, but the real issue is that RuboCop's `acceptable_range?` method checks whether the disabled cop is itself enabled in the project config. If the cop being disabled (`Layout/LineLength`) is already disabled by the `standard` gem, RuboCop skips the missing-enable offense. Implementing this requires cross-cop config access in nitrocop.

## Important Lessons Learned

### Cache Invalidation
nitrocop's stat-based result cache (`~/.cache/nitrocop/`) does NOT invalidate when cop code changes — only when target files change (mtime/size). During development:
1. Clear cache: `rm -rf ~/.cache/nitrocop`
2. Reinitialize lockfile: `cargo run --release -- --init ~/Dev/wt-gph-rspec-rip-out`
3. Use `--cache false` for all comparisons

### Prism AST Gotchas
- `begin X rescue Y end` (inline): `RescueModifierNode` inside `statements`, NOT `rescue_clause`
- `begin...end while true`: Prism creates `WhileNode` containing `BeginNode`
- Destructured params in `MultiTargetNode`: elements are `RequiredParameterNode`, not `LocalVariableTargetNode`
- `if cond; body else other end`: Prism returns `nil` for `then_keyword_loc` (semicolons serve as `then`)
- Method params named `it`: Prism uses `RequiredParameterNode` / `OptionalParameterNode`, not `LocalVariableWriteNode`

### RuboCop Config Nuances
- `Lint/MissingCopEnableDirective` has a cross-cop check: `acceptable_range?` skips offenses when the disabled cop is itself disabled in config.
- `Style/QuotedSymbols` resolves its style from `Style/StringLiterals EnforcedStyle` via `same_as_string_literals` — config inheritance from `inherit_gem: standard` may not resolve this correctly.

## Test Status

- **4656 library tests pass** (`cargo test --release --lib`)
- 7 integration test failures are pre-existing (unrelated)
- Clippy and fmt not yet run on session 3 changes

## Files Modified

```
src/cop/layout/argument_alignment.rs               |  16 +++-
src/cop/layout/array_alignment.rs                  |  26 ++++--
src/cop/layout/leading_comment_space.rs            |   7 +-
src/cop/layout/multiline_method_call_indentation.rs|   7 +-
src/cop/layout/multiline_operation_indentation.rs  |   9 +-
src/cop/lint/shadowed_exception.rs                 |   7 +-
src/cop/rails/http_positional_arguments.rs         |   5 +-
src/cop/style/comparable_clamp.rs                  | 102 ++++++++++++++++++-
src/cop/style/it_assignment.rs                     |  39 ++++----
src/cop/style/one_line_conditional.rs              |   2 -
src/cop/style/redundant_begin.rs                   |  65 +++++++++++++
src/cop/style/redundant_self.rs                    |  36 ++++++++
src/cop/style/slicing_with_range.rs                |  31 +++++++
src/cop/style/trailing_comma_in_arguments.rs       |  36 ++++++++
src/cop/style/trailing_comma_in_array_literal.rs   |   2 +-
src/cop/style/trailing_comma_in_hash_literal.rs    |   7 +-
src/linter.rs                                      |  20 +++-
tests/fixtures (14 fixture files)                  | +57 lines
```

## Next Steps (Suggested Priority)

1. **Run clippy/fmt, commit** — all 31 files are unstaged.
2. **Standard/BlockSingleLineBraces** (-23) — highest-impact unimplemented cop.
3. **Layout/IndentationWidth** (-6) — investigate tab handling without corpus regression.
4. **Layout/HashAlignment** (-3) — retry with more conservative approach.
5. **Style/Sample** (-3) — add `shuffle[n]` for non-zero n.
6. **Style/RedundantParentheses** (-3) — nested parentheses edge case.
7. **Style/RedundantRegexpEscape** (-2) — `\/` in `%r{}` and `\-` outside char class.
8. **Corpus fixture cleanup** — add semicolons between expressions in malformed single-line fixtures to eliminate 15 FPs from 5 cops.
