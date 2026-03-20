class A
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    def three
    end
  end
end

class B
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    def foo
    end

    def bar
    end
  end
end

class C
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    attr_reader :two

    def three
    end
  end
end

# private :new + def other — other is still public
class D
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    private :new

    def of_raw_data(site)
      42
    end
  end
end

# protected :new + def wrap — wrap is still public
class E
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    protected :new

    def wrap(o, c)
      42
    end
  end
end

# include + def — include doesn't affect visibility
class F
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    include Foo

    def bar
      42
    end
  end
end

# attr_reader + private :new + def — def is still public
class G
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    attr_reader :registered_plugins
    private :new

    def def_field(*names)
      42
    end
  end
end

# private :name before def name — def name redefines as public
class H
  class << self
  ^^^^^^^^^^^^^ Style/ClassMethodsDefinitions: Do not define public methods within class << self.
    private :next_migration_number

    def next_migration_number(dir)
      42
    end
  end
end

