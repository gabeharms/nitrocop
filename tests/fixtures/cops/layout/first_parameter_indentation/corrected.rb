def foo(
  bar,
  baz
)
end

def method_a(
  first,
  second
)
end

def method_b(
  first,
  second
)
end

# Keyword parameters should also be checked
def method_c(
  foo: 1,
  bar: 2
)
  foo
end

# Keyword-only params (no required params before them)
def method_d(
  name:,
  value:
)
  name
end

# Block parameter as first param
def method_e(
  &block
)
  block
end

# Keyword rest as first param
def method_f(
  **kwargs
)
  kwargs
end

# Post params (after rest) - rest is first param
def method_g(
  *args,
  post
)
  args
end

# def self.method (class method)
def self.method_h(
  first,
  second
)
  first
end
