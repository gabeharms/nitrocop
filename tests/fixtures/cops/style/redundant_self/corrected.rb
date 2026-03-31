def foo
  bar
end

def test
  to_s
end

def example
  method_name
end

class Foo
  def self.name_for_response
    name.demodulize
  end
end

class Bar
  def allowed(other)
    exists?(other)
  end
end
