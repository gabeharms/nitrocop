x = {
  a: 1,
  b: 2,
  c: 3
}

y = { a: 1, b: 2, c: 3 }

z = {
  foo: :bar,
  baz: :qux
}

# All elements on one line, only closing brace wraps — not an offense
ifaces = { 1 => {type: :hostonly, hostonly: "vboxnet0"}
         }

settings = { foo: :bar, baz: :qux
}

data = { key: "value"
}
