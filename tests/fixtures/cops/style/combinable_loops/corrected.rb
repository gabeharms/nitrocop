# Consecutive each loops
def test_consecutive
  items.each{ |item| do_something(item); do_something_else(item, arg) }
end

# Three consecutive loops
def test_three_consecutive
  items.each { |item| foo(item) }
  items.each { |item| bar(item) }
  items.each { |item| baz(item) }
end

# each_with_index
def test_each_with_index
  items.each_with_index{ |item| do_something(item); do_something_else(item, arg) }
end

# reverse_each
def test_reverse_each
  items.reverse_each{ |item| do_something(item); do_something_else(item, arg) }
end

# Blank lines between consecutive loops (no intervening code) — still an offense
def test_blank_lines
  items.each{ |item| alpha(item); beta(item) }
end

# for loops
def test_for_loops
  for item in items do do_something(item) end
  for item in items do do_something_else(item, arg) end
end

# each_with_object
def test_each_with_object
  items.each_with_object([]){ |item, acc| acc << item; acc << item.to_s }
end

# do...end blocks mixed with brace blocks
def test_do_end_blocks
  items.each do |item| do_something(item) end
  items.each { |item| do_something_else(item, arg) }
end

# Different block variable names — still an offense
def test_different_block_vars
  items.each { |item| foo(item) }
  items.each { |x| bar(x) }
end

# each_key
def test_each_key
  hash.each_key{ |k| do_something(k); do_something_else(k) }
end

# each_value
def test_each_value
  hash.each_value{ |v| do_something(v); do_something_else(v) }
end

# each_pair
def test_each_pair
  hash.each_pair{ |k, v| do_something(k); do_something_else(v) }
end

# Numbered block parameters
def test_numbered_blocks
  items.each { do_something(_1) }
  items.each { do_something_else(_1, arg) }
end
