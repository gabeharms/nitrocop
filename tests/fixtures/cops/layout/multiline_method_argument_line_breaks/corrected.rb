foo(
  bar,
  baz,
  qux
)

something(
  first,
  second,
  third
)

method_call(
  a,
  b,
  c
)

# Trailing keyword hash pairs sharing a line with positional args
taz("abc",
"foo",
"bar",
z: "barz",
x: "baz"
)

# Second arg starts on the same line as the end of multiline first arg
taz({
  foo: "edf",
},
   "abc")

# Hash arg starting on same line as positional arg
taz("abc",
  {
  foo: "edf",
})

# Safe navigation with args on multiple lines
foo&.bar(baz,
  quux,
  corge)

# Multiple args on first line
do_something(foo,
  bar,
  baz,
  quux)

# No-parens command call with keyword args on multiple lines
render json: data,
  status: :ok,
  layout: false

# No-parens command call with positional and keyword args
process records,
  format: :csv,
  output: path

# Block pass on same line as end of multiline hash arg
def message_area_tag(room, &)
  tag.div id: "message-area", class: "area", data: {
    controller: "messages",
  },
  &
end
