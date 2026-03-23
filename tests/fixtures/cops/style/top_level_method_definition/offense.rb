def foo
^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.
  'bar'
end

def baz
^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.
  42
end

def helper
^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.
  true
end

def self.class_method
^^^^^^^^^^^^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.
  false
end

define_method(:dynamic_foo) { puts 1 }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.

define_method(:dynamic_bar) do |x|
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.
  puts x
end

define_method(:dynamic_baz, instance_method(:foo))
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.

Foo.define_method(:receiver_method) { |*| nil }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.

Foo::Bar.define_method(:qualified_method) do
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/TopLevelMethodDefinition: Do not define methods at the top level.
  42
end
