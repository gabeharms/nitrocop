- `Lint/UnescapedBracketInRegexp`: token-local autocorrect by escaping offense-matched bare `]` as `\]` in regex content segments.
- `Lint/EmptyConditionalBody`: evaluate conservative autocorrect (`if cond; end` -> `if cond
  nil
end`) only if RuboCop parity and fixture safety are clear.
- `Naming/MemoizedInstanceVariableName`: consider a conservative subset that renames only simple `@ivar ||=` mismatches in method tail positions with same-scope reads.
- Keep Layout backlog staged (spacing-first, then alignment/indentation) until remaining non-Layout low-risk cops are exhausted.
