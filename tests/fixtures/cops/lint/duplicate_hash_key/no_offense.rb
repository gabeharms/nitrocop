hash = { a: 1, b: 2, c: 3 }
hash = { 'x' => 1, 'y' => 2, 'z' => 3 }
hash = { 1 => :a, 2 => :b, 3 => :c }
hash = {}
hash = { foo: 'bar' }
hash = { a: 1, **other }

# Method call keys are not literal duplicates
{ generate_id => "a", generate_id => "b" }
{ Time.now => "first", Time.now => "second" }
{ counter += 1 => "a", counter += 1 => "b" }
{ some_method_call(x, y) => 1, some_method_call(x, y) => 4 }

# [] calls on constants are not literal (could return different values)
{ Registry::Lookup['foo'] => FooHandler, Registry::Lookup['foo'] => BarHandler }

# Arithmetic expressions with +, -, / etc. are NOT literal per RuboCop
# (only ==, ===, !=, <, >, <=, >=, <=>, * are literal-preserving operators)
{ (2 * 3600 + 20 * 60) => 'first', (2 * 3600 + 20 * 60) => 'second' }
{ (10 - 3) => 'a', (10 - 3) => 'b' }
{ (10 / 2) => 'a', (10 / 2) => 'b' }
{ (10 % 3) => 'a', (10 % 3) => 'b' }
{ (2 ** 8) => 'a', (2 ** 8) => 'b' }
{ (1 << 4) => 'a', (1 << 4) => 'b' }
{ (16 >> 2) => 'a', (16 >> 2) => 'b' }
{ (5 & 3) => 'a', (5 & 3) => 'b' }
{ (5 | 3) => 'a', (5 | 3) => 'b' }
{ (5 ^ 3) => 'a', (5 ^ 3) => 'b' }
