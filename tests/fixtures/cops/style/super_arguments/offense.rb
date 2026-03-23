def foo(a, b)
  super(a, b)
  ^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

def bar(x, y)
  super(x, y)
  ^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

def baz(name)
  super(name)
  ^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

def with_rest(*args, **kwargs)
  super(*args, **kwargs)
  ^^^^^^^^^^^^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

def with_keyword(name:, age:)
  super(name: name, age: age)
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

def with_block(name, &block)
  super(name, &block)
  ^^^^^^^^^^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

def with_mixed(a, *args, b:)
  super(a, *args, b: b)
  ^^^^^^^^^^^^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

# super() with no-arg def
def no_args
  super()
  ^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

# super(...) forwarding
def with_forwarding(...)
  super(...)
  ^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

# Block-only forwarding
def block_only(&blk)
  super(&blk)
  ^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

# super with block literal should still flag when args match
def with_block_literal(a, b)
  super(a, b) { x }
  ^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

# super with block literal and non-forwarded block arg should flag
def with_block_arg_and_literal(a, &blk)
  super(a) { x }
  ^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when all positional and keyword arguments are forwarded.
end

# Ruby 3.1 hash value omission: super(type:, hash:)
def with_shorthand_keywords(type:, hash:)
  super(type:, hash:)
  ^^^^^^^^^^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end

# Nested def — inner super should flag for inner def
def outer(a)
  def inner(b:)
    super(b: b)
    ^^^^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
  end
  super(a)
  ^^^^^^^^ Style/SuperArguments: Call `super` without arguments and parentheses when the signature is identical.
end
