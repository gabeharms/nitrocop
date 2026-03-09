def badMethod
    ^^^^^^^^^ Naming/MethodName: Use snake_case for method names.
  x = 1
end

def another_Bad_one
    ^^^^^^^^^^^^^^^ Naming/MethodName: Use snake_case for method names.
  y = 2
end

def myHelper
    ^^^^^^^^ Naming/MethodName: Use snake_case for method names.
  nil
end

# Singleton camelCase method (lowercase start) should be flagged
def self.myMethod
         ^^^^^^^^ Naming/MethodName: Use snake_case for method names.
end

# attr_reader with camelCase symbol
attr_reader :myMethod
            ^^^^^^^^^ Naming/MethodName: Use snake_case for method names.

# attr_accessor with camelCase symbol
attr_accessor :myMethod
              ^^^^^^^^^ Naming/MethodName: Use snake_case for method names.

# attr_writer with camelCase symbol
attr_writer :myMethod
            ^^^^^^^^^ Naming/MethodName: Use snake_case for method names.

# define_method with camelCase symbol
define_method :fooBar do
              ^^^^^^^ Naming/MethodName: Use snake_case for method names.
end

# define_singleton_method with camelCase symbol
define_singleton_method :fooBar do
                        ^^^^^^^ Naming/MethodName: Use snake_case for method names.
end

# alias with camelCase
alias fooBar foo
      ^^^^^^ Naming/MethodName: Use snake_case for method names.

# alias_method with camelCase symbol
alias_method :fooBar, :foo
             ^^^^^^^ Naming/MethodName: Use snake_case for method names.

# SCREAMING_CASE singleton methods should be flagged (not CamelCase factory exceptions)
def self.IF(test, then_e, else_e)
         ^^ Naming/MethodName: Use snake_case for method names.
end

def self.UNLESS(test, then_e, else_e)
         ^^^^^^ Naming/MethodName: Use snake_case for method names.
end

# CamelCase singleton factory methods are also flagged by RuboCop
def self.Dimension(*args)
         ^^^^^^^^^ Naming/MethodName: Use snake_case for method names.
  new(*args)
end
