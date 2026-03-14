foo { |a, b| a + b }
bar { |x| x }
baz { |item| item.to_s }
qux { puts 'hello' }
items.each { |i| puts i }
x = [1, 2, 3]

# Single parameter with trailing comma (destructuring) — NOT an offense
items.each { |item,| process(item) }
hash.each_key { |key,| keys << key }
[1, 2].map { |x, | x * 2 }
test { |a,| a }
test do |a,|
  a
end
define_method(:m) { |a,| a }

# Chained blocks: inner block has single-param trailing comma,
# outer block has 2+ params. RuboCop incorrectly flags these
# due to a bug in argument_tokens (pipe-matching across blocks).
# nitrocop correctly does NOT flag.
items.sort_by { |name,| name }.map do |name, value|
  value.to_s
end
data.select { |k,| allowed?(k) }.each do |k, v|
  process(k, v)
end
groups.sort_by do |day,| day end.reverse_each do |day, entries|
  display(day, entries)
end
