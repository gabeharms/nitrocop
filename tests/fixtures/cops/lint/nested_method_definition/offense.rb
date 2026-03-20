def foo
  def bar
  ^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
    something
  end
end
def baz
  def qux
  ^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
    other
  end
end
def outer
  def inner
  ^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
    42
  end
end

# def self.method inside another def IS an offense (self is not an allowed receiver)
class Foo
  def self.x
    def self.y
    ^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
    end
  end
end

# def inside a lambda block is still an offense
def foo
  bar = -> { def baz; puts; end }
             ^^^^^^^^^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
end

# def inside a random block is still an offense
def do_something
  items.each do
    def process_item
    ^^^^^^^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
    end
  end
end

# def inside a class inside a def is still an offense (class is NOT scope-creating per RuboCop)
def bar
  class MyClass
    def inner_method
    ^^^^^^^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
      work
    end
  end
end

# def inside a module inside a def is still an offense
def baz
  module MyModule
    def inner_method
    ^^^^^^^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
      work
    end
  end
end

# Parenthesized receiver with assignment — NOT an allowed receiver type
# In Parser gem, (lvasgn ...) is the receiver, which is not variable?/const_type?/call_type?
def test_method
  def (obj = Object.new).helper = true
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
end

# Parenthesized safe-navigation receiver — NOT an allowed receiver type
# In Parser gem, (begin (csend ...)) is the receiver; begin is not call_type?
class Foo
  def x
    def (do_something&.y).z
    ^^^^^^^^^^^^^^^^^^^^^^^ Lint/NestedMethodDefinition: Method definitions must not be nested. Use `lambda` instead.
    end
  end
end
