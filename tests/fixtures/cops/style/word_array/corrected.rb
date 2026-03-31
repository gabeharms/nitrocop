%w[foo bar baz]

%w[one two]

x = %w[alpha beta gamma]

# Hyphenated words should be flagged (matches default WordRegex)
%w[foo bar foo-bar]

# Unicode word characters should be flagged
%w[hello world café]

# Strings with newline/tab escapes are words per default WordRegex
["one\n", "hi\tthere"]

# Matrix where all subarrays are simple words — each subarray still flagged
[
  %w[one two],
  %w[three four]
]

# Parenthesized call with block is NOT ambiguous — should still flag
foo(%w[bar baz]) { qux }

# Matrix with mixed-type subarrays — pure-string subarrays still flagged
# (non-string elements like 0 don't make a subarray "complex" for matrix check)
[["foo", "bar", 0], %w[baz qux]]
