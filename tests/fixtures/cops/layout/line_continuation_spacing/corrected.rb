one_plus_one = 1 \
  + 1

two_plus_two = 2 \
  + 2

three = 3 \
  + 0

# String followed by backslash with no space (implicit concatenation)
msg = "Expected result " \
    "but found something else"

raise TypeError, "Argument must be a Binding, not " \
    "a #{scope.class.name}"

result = 'hello ' \
    'world'

x = 'first' \
    'second'

# Multiple spaces before backslash after string
msg2 = "hello" \
    "world"

# Method call with backslash continuation, no space
expect { foo }.to \
  raise_error(ArgumentError)

# Multi-line condition, no space before backslash
if condition_a && \
   condition_b
  do_something
end

# Quoted symbol with escaped backslash spanning lines — RuboCop flags this
# because :sym nodes are not in ignored_literal_ranges
x = :"a\ \
b"
