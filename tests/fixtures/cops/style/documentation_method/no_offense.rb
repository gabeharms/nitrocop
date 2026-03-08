# Public method
def foo
  puts 'bar'
end

def initialize
  @x = 1
end

# Another documented method
def bar
  42
end

# Private methods don't need docs (default RequireForNonPublicMethods: false)
private

def private_method
  42
end

protected

def protected_method
  42
end

# Inline private
private def inline_private
  42
end

# Documentation for modular method
module_function def modular_method
  42
end

# Documentation for keywords method
ruby2_keywords def keyword_method
  42
end

# private_class_method is non-public, skipped by default
private_class_method def self.secret
  42
end

# TODO: fix this
# Real documentation follows the annotation
def annotated_then_doc
  42
end

# Private with indented def (common Ruby style)
class IndentedPrivate
  private
    def indented_private_method
      42
    end

  protected
    def indented_protected_method
      42
    end
end

# Private inside class << self followed by private section
module ActionCable
    class Base
      class << self
      end
      private
        def delegate_connection_identifiers
          42
        end
    end
end

# Private in nested class with different indentation
class Container
  class Nested
    private
      def deeply_nested_private
        42
      end
  end
end

# Retroactive private :method_name makes method non-public (no docs needed)
class RetroactivePrivate
  def secret_method
    42
  end
  private :secret_method
end

# Retroactive protected :method_name makes method non-public
class RetroactiveProtected
  def guarded_method
    42
  end
  protected :guarded_method
end

# Multiple methods made private retroactively
class MultiRetroactive
  def helper_one
    42
  end

  def helper_two
    42
  end
  private :helper_one, :helper_two
end

# Retroactive private with string argument
class RetroactivePrivateString
  def string_method
    42
  end
  private "string_method"
end

# public re-establishes visibility after private section
class PublicAfterPrivate
  private

  def secret
    42
  end

  public

  # Documented public method after public keyword
  def visible
    42
  end
end
