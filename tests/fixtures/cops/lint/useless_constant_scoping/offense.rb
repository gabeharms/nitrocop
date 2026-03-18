class Foo
  private
  PRIVATE_CONST = 42
  ^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
end

class Bar
  private
  MY_CONST = 'hello'
  ^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
end

class Baz
  private
  X = 1
  ^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
end

class Provider
  private

  self::QUERY_FORMAT = "'${Status} ${Package}\\n'"
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
  self::FIELDS_REGEX = /^(\S+) +(\S+)$/
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
  self::FIELDS = [:name, :status]
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
end

# Singleton class (class << self)
module Helpers
  class << self
    private

    TARGETS = [:both, :enforced, :report_only]
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
  end
end

class Config
  class << self
    private

    SNAKE_CASE = ->(word) { word.downcase }
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.

    DEFAULT_PRIORITY = 0
    ^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
  end
end

# DSL block pattern
Something.provide :handler do
  private

  self::FORMAT_STRING = "'${Status}\\n'"
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
  self::FIELDS_REGEX = /^(\S+) +(\S+)$/
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
end

# Constant after private with methods in between
class Service
  class << self
    private

    def something; end

    PATTERN = /foo/
    ^^^^^^^^^^^^^^^ Lint/UselessConstantScoping: Useless `private` access modifier for constant scope.
  end
end
