# frozen_string_literal: false
a = Array.new
    ^^^^^^^^^ Style/EmptyLiteral: Use array literal `[]` instead of `Array.new`.

h = Hash.new
    ^^^^^^^^ Style/EmptyLiteral: Use hash literal `{}` instead of `Hash.new`.

s = String.new
    ^^^^^^^^^^ Style/EmptyLiteral: Use string literal `''` instead of `String.new`.

# Inner Hash.new inside a Hash.new block should still be flagged
@ini = Hash.new {|h,k| h[k] = Hash.new}
                              ^^^^^^^^ Style/EmptyLiteral: Use hash literal `{}` instead of `Hash.new`.

# Inner Array.new inside Array.new block should still be flagged
rows = Array.new(row_count) { Array.new }
                              ^^^^^^^^^ Style/EmptyLiteral: Use array literal `[]` instead of `Array.new`.

columns = Array.new(option) { Array.new }
                              ^^^^^^^^^ Style/EmptyLiteral: Use array literal `[]` instead of `Array.new`.
