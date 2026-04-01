do_something if x

do_something unless x

foo if condition

retry unless finished?

# Parenthesized condition (non-assignment) should still be flagged
do_something if (x > 0)

# Blank line between condition and body should still be flagged
do_something if condition

# One-line form should be flagged
bar if foo
