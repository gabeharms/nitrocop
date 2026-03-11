def foo
  super(a, b)
end

def bar
  super
end

def baz
  super()
end

def qux(&block)
  super(&block)
end

x = 1
y = 2
