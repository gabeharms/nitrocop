class Foo
  attr_accessor :bar
  other_macro :something
end

class Baz
  attr_accessor :qux
end

# Same visibility scope (both public, then both private)
class SameVisibility
  attr_accessor :foo

  private

  attr_accessor :baz
end

# Within eigenclass
class WithEigenclass
  attr_reader :bar

  class << self
    attr_accessor :baz

    private

    attr_reader :quux
  end
end

module SomeModule
  attr_reader :name
  attr_writer :name, :role
end
