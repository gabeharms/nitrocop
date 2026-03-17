def initialize
  # initializer comment
end

def initialize(a, b)
  do_something
end

def initialize(a, b)
  super
  do_something
end

def do_something
end

def initialize(a, b)
  super()
end

def initialize(a, b = 5)
  super
end

def initialize(*args)
  super
end

def initialize(**kwargs)
  super
end

# Empty initialize with parameter — not redundant (overrides parent)
def initialize(_assistant); end
def initialize(arg)
end

# Inline comment on def line — allowed with AllowComments: true (default)
def initialize # some comment
  super
end

# super with different number of args — not redundant
def initialize(a, b)
  super(a)
end

# super with different arg names — not redundant
def initialize(a, b)
  super(b, a)
end

# super with extra args — not redundant
def initialize(a)
  super(a, b)
end
