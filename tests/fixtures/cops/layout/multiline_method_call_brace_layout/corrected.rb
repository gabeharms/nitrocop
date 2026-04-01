foo(a,
  b)

bar(
  a,
  b
)

baz(c,
  d)

foo(<<~EOS, arg
  text
EOS
).do_something
