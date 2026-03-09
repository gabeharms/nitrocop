def good_method
  x = 1
end

def initialize
  @x = 1
end

def <=>(other)
  x <=> other
end

def <<(item)
  items << item
end

def []=(key, value)
  @hash[key] = value
end

def _private_method
  nil
end

def save!
  true
end

def valid?
  true
end

# Unicode characters in method names (not ASCII uppercase)
def elapsed_μs
  42
end

def nowµs
  Time.now
end

# attr_reader with snake_case is fine
attr_reader :my_method
attr_accessor :my_method
attr_writer :my_method

# define_method with snake_case is fine
define_method :foo_bar do
end

# define_method with operator is fine
define_method :== do
end

define_method :[] do
end

# define_method without arguments is fine
define_method do
end

# define_method with variable (not literal) is fine
define_method foo do
end

# alias with snake_case is fine
alias foo_bar baz

# alias_method with snake_case is fine
alias_method :foo_bar, :baz

# alias_method with non-symbol first arg is fine
alias_method foo, :bar

# alias_method with wrong arity is fine
alias_method :fooBar, :bar, :baz
