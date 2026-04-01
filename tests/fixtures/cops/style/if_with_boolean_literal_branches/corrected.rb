foo == bar

foo.do_something?

!(foo.do_something?)

foo.do_something?

foo == bar

# elsif with boolean literal branches
if foo
  true
elsif bar.empty?
  true
else
  false
end

# Complex || chain of predicate methods
(a.present? || b.present? || c.present?)

# and keyword with boolean-returning right operand
(foo and bar == "baz")

# Predicate method with block_pass argument (&:sym) — should be flagged
# In Prism, &:sym is a BlockArgumentNode, NOT a BlockNode.
# RuboCop treats `all?(&:fulfilled?)` as a send node (predicate method).
futures.all?(&:fulfilled?)
