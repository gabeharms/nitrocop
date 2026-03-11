x = 'hello #{name}'
    ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.

y = 'value: #{foo}'
    ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.

z = '#{bar}'
    ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.

w = 'THIS. IS. #{yield.upcase}!'
    ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.

# String containing double quotes — RuboCop flags this (corrects with %{})
a = 'foo "#{bar}"'
    ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.

# Split string where the second part has interpolation
"x" \
  'foo #{bar}'
  ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.

# %q string with inner double quotes — RuboCop flags this
# (RuboCop's valid_syntax? always passes for %q since gsub doesn't change it)
b = %q{text "#{name}"}
    ^ Lint/InterpolationCheck: Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.
