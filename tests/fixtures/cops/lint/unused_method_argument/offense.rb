def some_method(used, unused)
                      ^^^^^^ Lint/UnusedMethodArgument: Unused method argument - `unused`.
  puts used
end

def foo(bar, baz)
             ^^^ Lint/UnusedMethodArgument: Unused method argument - `baz`.
  bar
end

def calculate(x, y, z)
                    ^ Lint/UnusedMethodArgument: Unused method argument - `z`.
  x + y
end

def protect(*args)
             ^^^^ Lint/UnusedMethodArgument: Unused method argument - `args`.
  do_something
end

# block parameter unused
def with_block(a, &block)
                   ^^^^^ Lint/UnusedMethodArgument: Unused method argument - `block`.
  puts a
end

# keyword rest parameter unused
def with_kwrest(a, **opts)
                     ^^^^ Lint/UnusedMethodArgument: Unused method argument - `opts`.
  puts a
end

# post parameter unused (after rest)
def with_post(*args, last)
                     ^^^^ Lint/UnusedMethodArgument: Unused method argument - `last`.
  args.first
end

# multi-assign target only (not a read)
def multi_target(a, b)
                 ^ Lint/UnusedMethodArgument: Unused method argument - `a`.
                    ^ Lint/UnusedMethodArgument: Unused method argument - `b`.
  a, b = 1, 2
end
