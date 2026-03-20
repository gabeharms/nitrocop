class Foo
  include Bar
  ^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  attr_reader :baz
end

class Qux
  extend ActiveSupport::Concern
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  def some_method
  end
end

class Abc
  prepend MyModule
  ^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  def another_method
  end
end

# include inside multi-statement block (Class.new, RSpec.describe, etc.)
Class.new do
  include AccountableConcern
  ^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  attr_reader :current_account
  def initialize
  end
end

RSpec.describe User do
  include RSpec::Rails::RequestExampleGroup
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  let(:username) { 'alice' }
  it 'does something' do
  end
end

# include inside class nested within if block (class resets if context)
if some_condition
  class Child
    include Serializable
    ^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    attr_reader :data
  end
end

require "support/helpers"

include Support::Helpers
^^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
records = build_records

def setup
  include MyHelper
  ^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  do_stuff
end

# include inside multi-statement if body (parent is begin, not if)
class Config
  if RUBY_VERSION >= '1.9'
    include Comparable
    ^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    def <=>(other)
      name <=> other.name
    end
  end
end

# extend inside multi-statement if body
class Worker
  if feature_enabled?
    extend Forwardable
    ^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    def_delegator :config, :timeout
  end
end

# include inside begin...rescue block
class Service
  begin
    include Serializable
    ^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    validate :check_format
  rescue NameError
    use_fallback
  end
end

# include with rescue modifier followed by non-include code
class Provider
  include Logging rescue LoadError
  ^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  validate :check_config
end

# extend with rescue modifier followed by non-include code
module Helpers
  extend Formatting rescue NameError
  ^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  def setup; end
end

# include before rescue clause still needs a separating blank line
begin
  extend DynamicBehavior
  ^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
rescue NameError
  use_fallback
end

# rescue-modified include breaks the inclusion group
class Legacy
  include Serializable rescue NameError
  ^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  include Comparable

  def setup; end
end

# rescue-modified include is not grouped with adjacent includes on either side
class Windows
  include ShellOut
  ^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  include Error rescue LoadError
  ^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  include Constants

  AUTO_START = "auto".freeze
end

# include inside a nested block within a single-statement if body should still be checked
if windows?
  do_work do
    mod = Module.new do
      extend FFI::Library
      ^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
      ffi_lib "c"
    end

    mod.ffi_libraries
  end
end

# single-statement blocks should not suppress nested begin/rescue bodies
klass.class_eval do
  begin
    include Submodule
    ^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
  rescue NameError
  end
end

# single-statement blocks should not suppress nested if bodies
included do
  if defined?(ActionController::StrongParameters)
    include ActionController::StrongParameters
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    map_error! ActionController::ParameterMissing, RocketPants::BadRequest
  end
end

# single-statement blocks should not suppress nested def bodies
class_methods do
  def enable_url_helpers
    include Rails.application.routes.url_helpers
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    Rails.application.routes.default_url_options[:host] = "example.com"
  end
end

# single-statement if bodies should not suppress nested begin/rescue bodies
if !ENV["CI"]
  begin
    include SimpleCovHelper
    ^^^^^^^^^^^^^^^^^^^^^^^ Layout/EmptyLinesAfterModuleInclusion: Add an empty line after module inclusion.
    start_simple_cov("suite")
  rescue LoadError
  end
end
