class Foo
  public
  ^^^^^^ Lint/UselessAccessModifier: Useless `public` access modifier.

  def method
  end
end

class Bar
  private
  ^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.
end

class Baz
  protected
  ^^^^^^^^^ Lint/UselessAccessModifier: Useless `protected` access modifier.
end

module Qux
  private
  ^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.

  def self.singleton_method
  end
end

# private_class_method without arguments is useless
class WithPrivateClassMethod
  private_class_method
  ^^^^^^^^^^^^^^^^^^^^ Lint/UselessAccessModifier: Useless `private_class_method` access modifier.

  def self.calculate_something(data)
    data
  end
end

# top-level access modifiers are always useless
private
^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.

def top_level_method
end

protected
^^^^^^^^^ Lint/UselessAccessModifier: Useless `protected` access modifier.

def another_top_level_method
end

# module_function at top level is useless
module_function
^^^^^^^^^^^^^^^ Lint/UselessAccessModifier: Useless `module_function` access modifier.

def top_func
end

# module_function inside a module followed only by eval is useless
module WithModuleFunction
  module_function
  ^^^^^^^^^^^^^^^ Lint/UselessAccessModifier: Useless `module_function` access modifier.
  eval "def test1() end"
end

# module_function repeated inside a module
module RepeatedModuleFunction
  module_function

  def first_func; end

  module_function
  ^^^^^^^^^^^^^^^ Lint/UselessAccessModifier: Useless `module_function` access modifier.

  def second_func; end
end

# useless access modifier inside Class.new do block
Class.new do
  private
  ^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.
end
