class Foo
  private

  def bar
    puts 'bar'
  end

  protected

  def baz
    puts 'baz'
  end

  private :some_method

  # Visibility-change calls (not inline modifier declarations)
  public target
  private method_var
  protected some_method_name
end

# Access modifiers inside block bodies should be ignored
# (RuboCop only checks inside class/module/sclass bodies)
module Pakyow
  class Application
    class_methods do
      private def load_aspect(aspect)
        aspect.to_s
      end

      protected def another_method
        true
      end
    end
  end
end

class SomeService
  included do
    private def helper
      'help'
    end
  end
end

# Inside a regular block (not class/module body)
concern do
  private def perform
    run
  end
end

# Conditional access modifiers should be skipped
# (RuboCop skips when parent is an if/unless node)
class ConditionalModifier
  if some_condition
    private :foo
  end

  unless other_condition
    protected :bar
  end
end

# Inline modifier inside conditional (parent is if_type)
class ConditionalInline
  if some_flag
    private def secret_method
      'secret'
    end
  end
end
