[1, 2, 3].include?(e)
{ foo: :bar }.key?(:foo)
loop do
  [1, nil, 3].compact!
end
loop do
  { foo: :bar, baz: nil }.select! { |_k, v| !v.nil? }
end
loop do
  array = [1, nil, 3]
end
loop do
  hash = { foo: :bar, baz: nil }
end
[1, 2, variable].include?(e)
{ foo: { bar: variable } }.key?(:foo)
[[1, 2, 3] | [2, 3, 4]].each { |e| puts e }
loop do
  array.all? { |x| x > 100 }
end
while x < 100
  puts x
end
# Safe navigation (&.) should NOT be treated as a loop
items&.each { |item| [1, 2, 3].include?(item) }
items&.map { |item| [1, 2, 3].include?(item) }
items&.select { |item| { a: 1 }.key?(item) }
# Interpolated regex is NOT a basic literal — should not be flagged
items.each { |item| [/foo/, /#{item}/].any? { |r| "str".match?(r) } }
