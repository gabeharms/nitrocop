# No parent class - no super needed
class Child
  def initialize
    do_something
  end
end

# Calls super
class Child < Parent
  def initialize
    super
    do_something
  end
end

# Stateless parent Object
class Child < Object
  def initialize
    do_something
  end
end

# Stateless parent BasicObject
class Child < BasicObject
  def initialize
    do_something
  end
end

# Class.new without parent
Class.new do
  def initialize
    do_something
  end
end

# Class.new with stateless parent
Class.new(Object) do
  def initialize
    do_something
  end
end

# Module - not a class
module M
  def initialize
    do_something
  end
end

# Callback with super
class Foo
  def self.inherited(base)
    super
    do_something
  end
end

# method_added with super
class Foo
  def method_added(name)
    super
    do_something
  end
end

# FP fix: def initialize inside a non-Class.new block within a class with parent
# RuboCop checks nearest block ancestor first — if it's not Class.new(Parent), no offense
class Child < Parent
  some_method do
    def initialize
      do_something
    end
  end
end

# FP fix: Class.new(Parent) with intervening non-Class.new block
Class.new(Parent) do
  items.each do
    def initialize
      do_something
    end
  end
end

# FP fix: Class.new without parent inside class with parent
class Child < Parent
  Class.new do
    def initialize
    end
  end
end

# FP fix: def self.initialize is a class method, not a constructor — only instance
# initialize should trigger the offense (RuboCop on_defs does not check initialize)
class Child < Parent
  def self.initialize
    do_something
  end
end

# FP fix: lambda block acts as barrier for initialize check
# RuboCop checks each_ancestor(:any_block).first — lambda counts as block
class Child < Parent
  validator = lambda do
    def initialize
      do_something
    end
  end
end

# FP fix: arrow lambda also acts as barrier
class Child < Parent
  validator = -> do
    def initialize
      do_something
    end
  end
end

# FP fix: RuboCop's contains_super? uses each_descendant which traverses into
# nested defs/classes/modules — super in a nested scope counts
class Child < Parent
  def initialize
    klass = Class.new do
      def setup
        super
      end
    end
  end
end

# FP fix: super inside a nested class still counts for RuboCop
class Child < Parent
  def initialize
    class Inner < Base
      def setup
        super
      end
    end
  end
end

# FP fix: super inside a nested module still counts for RuboCop
class Child < Parent
  def initialize
    module Helper
      def setup
        super
      end
    end
  end
end
