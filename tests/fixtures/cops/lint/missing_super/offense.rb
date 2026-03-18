class Child < Parent
  def initialize
  ^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to initialize state of the parent class.
  end
end

class Child < Parent
  def initialize(name, salary)
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to initialize state of the parent class.
    @salary = salary
  end
end

Class.new(Parent) do
  def initialize
  ^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to initialize state of the parent class.
  end
end

class Foo
  def self.inherited(base)
  ^^^^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to invoke callback defined in the parent class.
  end
end

class Foo
  def method_added(*)
  ^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to invoke callback defined in the parent class.
  end
end

class Foo
  class << self
    def inherited(base)
    ^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to invoke callback defined in the parent class.
    end
  end
end

# FN fix: callback inside a module should also be flagged
module Callbacks
  def method_added(name)
  ^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to invoke callback defined in the parent class.
  end
end

module Hooks
  def self.inherited(base)
  ^^^^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to invoke callback defined in the parent class.
  end
end

module Extensions
  class << self
    def singleton_method_added(name)
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to invoke callback defined in the parent class.
    end
  end
end

# FN fix: qualified constant path superclass (e.g. Synchronization::Object)
# should NOT match stateless "Object" — full name must be compared
class Record < Synchronization::Object
  def initialize(value)
  ^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to initialize state of the parent class.
    @value = value
  end
end

# FN fix: qualified constant path superclass nested in modules
module Concurrent
  class Container < Synchronization::Object
    def initialize(opts = {})
    ^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to initialize state of the parent class.
      @opts = opts
    end
  end
end

# FN fix: deeply qualified constant path superclass
class Handler < Utils::Sync::BasicObject
  def initialize(name)
  ^^^^^^^^^^^^^^^^^^^^ Lint/MissingSuper: Call `super` to initialize state of the parent class.
    @name = name
  end
end
