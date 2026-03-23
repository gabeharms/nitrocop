class Foo
  def bar
    'baz'
  end
end

module Helper
  def help
    true
  end
end
x = 1

class Foo
  define_method(:a) { puts 1 }

  define_method(:b) do |x|
    puts x
  end

  define_method(:c, instance_method(:d))
end

Foo = Struct.new do
  def some_method; end
end

require_relative 'foo'
