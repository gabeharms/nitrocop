def m(&block)
  items.something(&block)
end

def n
  items.something { |i, j| yield j, i }
end

def o
  items.something { |i| do_something(i) }
end

def p
  yield
end

# Block with yield outside a method definition - not an offense
render("partial") do
  yield
end

items.each do |x|
  yield x
end

collection.map { |item| yield item }

# Destructured block params should not be flagged
def normalize_repeatable_value(option_name, value)
  value.each do |(key, val)|
    yield [[option_name, key], val]
  end
end

def stream(tokens)
  formatted_lines.each {|(lineno, line)| yield line }
end

# Block with & parameter should not be flagged
def create
  Proc.new do |&b|
    yield
  end
end

# Block with * rest parameter should not be flagged
def wrap
  items.each do |*args|
    yield args.first
  end
end

# Block with ** keyword rest parameter should not be flagged
def wrap2
  items.each do |**opts|
    yield opts
  end
end

# Lambda with non-yield body should not be flagged
def t
  metric.time(name, -> { do_something })
end

# Lambda with yield outside method def should not be flagged
-> { yield }.call

# Lambda with args that don't match yield should not be flagged
def u
  metric.time(name, ->(x) { yield })
end
