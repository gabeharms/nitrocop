class Foo
  def bar; end
end

class Bar
  X = 1
end

class Baz; end

class << self
  def foo
  end
end

class Qux
  class << self
    def bar
    end
  end
end

class MultilineParent <
  BaseClass
  def method
  end
end
