# Simple operator conditions — block form (keyword and node start are the same)
if x == y
  do_something
end
if foo.odd?
  bar
end

# Simple operator conditions — modifier form (node starts at body, not keyword)
do_something if x <= 0

# Negation with !
foo if bar
foo if !bar

# Methods without explicit receiver (implicit self)
foo if even?

# Complex compound conditions (AND/OR)
foo if x == y || x.even?
foo if x == y && x.odd?

# Parenthesized conditions
foo if ((x == y))

# Other invertible operators
do_something if x < 10
do_something if x > 5
do_something if x >= 3
do_something if x =~ /pattern/

# Predicate methods
foo if items.nonzero?
foo if items.none?
foo if items.any?
foo if items.zero?

# Complex nested compound condition
foo if x == y || (((x.even?) && (((y < 5)))) && z.nonzero?)

# All-uppercase constant with < is NOT inheritance (so it IS invertible)
foo if x >= FOO

# Multi-line modifier unless — offense reported at start of expression (line 1), not at keyword
errors.add(
  :base,
  "subject and expected_receive_period_in_days are required"
) if x == y
