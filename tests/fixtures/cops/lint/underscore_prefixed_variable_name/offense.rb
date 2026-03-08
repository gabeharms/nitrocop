# Method parameter with underscore prefix that is used
def some_method(_bar)
                ^^^^ Lint/UnderscorePrefixedVariableName: Do not use prefix `_` for a variable that is used.
  puts _bar
end

# Optional parameter with underscore prefix that is used
def another_method(_baz = 1)
                   ^^^^ Lint/UnderscorePrefixedVariableName: Do not use prefix `_` for a variable that is used.
  puts _baz
end

# Block parameter with underscore prefix that is used
items.each do |_item|
               ^^^^^ Lint/UnderscorePrefixedVariableName: Do not use prefix `_` for a variable that is used.
  puts _item
end

# Lambda parameter with underscore prefix that is used
handler = ->(_event) do
             ^^^^^^ Lint/UnderscorePrefixedVariableName: Do not use prefix `_` for a variable that is used.
  process(_event)
end

# Local variable assignment with underscore prefix that is used
def process_data
  _result = compute
  ^^^^^^^ Lint/UnderscorePrefixedVariableName: Do not use prefix `_` for a variable that is used.
  _result.save
end

# Top-level local variable with underscore prefix that is used
_top = 1
^^^^ Lint/UnderscorePrefixedVariableName: Do not use prefix `_` for a variable that is used.
puts _top
