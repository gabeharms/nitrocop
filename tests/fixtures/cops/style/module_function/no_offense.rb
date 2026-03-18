module Test
  module_function
  def test; end
end
class Foo
  extend self
end
module Bar
  extend SomeModule
end
module Baz
  module_function :test
end
# extend self with private directive should not be flagged
module WithPrivate
  extend self
  def greet; end
  private
  def helper; end
end
# extend self with private :method_name should not be flagged
module WithPrivateMethod
  extend self
  def greet; end
  private :helper
  def helper; end
end
# extend self with private def should not be flagged
module WithPrivateDef
  extend self
  def greet; end
  private def helper; end
end
# extend self as the only statement — RuboCop requires begin_type? (2+ statements)
module SelfExtendingOnly
  extend self
end
# nested module with extend self as only statement (corpus FP pattern)
module SingletonMethodsSpecs
  module Prepended
    def mspec_test_kernel_singleton_methods
    end
    public :mspec_test_kernel_singleton_methods
  end

  ::Module.prepend Prepended

  module SelfExtending
    extend self
  end
end
