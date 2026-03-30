"#{foo} bar"

"#{foo} bar"

"#{foo} bar"

foo(<<~MSG)
  foo #{bar}
  baz
MSG

<<~MSG
  foo #{bar}
  baz
MSG
