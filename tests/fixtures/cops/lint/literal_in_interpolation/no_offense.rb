"result is #{compute}"
"hello #{name}"
"#{x + y}"
"#{foo(bar)}"
"no interpolation"
"#{variable}"

# Whitespace-only string literals at end of heredoc line are allowed
# (used for trailing whitespace preservation with Layout/TrailingWhitespace)
x = <<~MSG
  Add the following:#{' '}
MSG

# Interpolation of non-literal ranges is allowed
"this is an irange: #{var1..var2}"
"this is an erange: #{var1...var2}"

# Special keywords are not literals
%("this is #{__FILE__} silly")
%("this is #{__LINE__} silly")
%("this is #{__ENCODING__} silly")

# Interpolation of xstr is allowed
"this is #{`hostname`} silly"

# Arrays inside regexp are handled by Lint/ArrayLiteralInRegexp
/#{%w[a b c]}/

# Heredoc interpolation is allowed
"#{<<~TEXT}"
hello
TEXT
