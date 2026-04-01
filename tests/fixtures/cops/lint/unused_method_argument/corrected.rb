def some_method(used, _unused)
  puts used
end

def foo(bar, _baz)
  bar
end

def calculate(x, y, _z)
  x + y
end

def protect(*_args)
  do_something
end

# block parameter unused
def with_block(a, &_block)
  puts a
end

# keyword rest parameter unused
def with_kwrest(a, **_opts)
  puts a
end

# post parameter unused (after rest)
def with_post(*args, _last)
  args.first
end

# multi-assign target only (not a read)
def multi_target(_a, _b)
  a, b = 1, 2
end

# block parameter shadows method parameter — method param is unused
def shadowed_by_block(_x)
  items.each { |x| puts x }
end

# lambda parameter shadows method parameter — method param is unused
def shadowed_by_lambda(_x)
  transform = ->(x) { x * 2 }
  transform.call(42)
end

# binding(&block) is NOT Kernel#binding — does not suppress unused arg warning
def with_binding_block_pass(_bar, &blk)
  binding(&blk)
end
