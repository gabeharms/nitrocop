describe SomeClass do
  CONSTANT = "Accessible as ::CONSTANT".freeze
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
end

describe SomeClass do
  class DummyClass < described_class
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub class constant instead of declaring explicitly.
  end
end

describe SomeClass do
  module DummyModule
  ^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub module constant instead of declaring explicitly.
  end
end

RSpec.shared_examples 'shared example' do
  CONSTANT = "Accessible as ::CONSTANT".freeze
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
end

describe SomeClass do
  specify do
    CONSTANT = "Accessible as ::CONSTANT".freeze
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
  end
end

# Constants nested inside control structures should still be flagged
describe SomeClass do
  if some_condition
    NESTED_CONST = "leaky"
    ^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
  end
end

describe SomeClass do
  unless some_condition
    class NestedClass
    ^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub class constant instead of declaring explicitly.
    end
  end
end

describe SomeClass do
  case something
  when :foo
    module NestedModule
    ^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub module constant instead of declaring explicitly.
    end
  end
end

describe SomeClass do
  begin
    RESCUE_CONST = "leaky"
    ^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
  rescue StandardError
    nil
  end
end

# ConstantOrWriteNode (CONST ||= val)
describe SomeClass do
  FALLBACK ||= "default"
  ^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
end

# ConstantAndWriteNode (CONST &&= val)
describe SomeClass do
  FLAG &&= false
  ^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
end

# ConstantOperatorWriteNode (CONST += val)
describe SomeClass do
  COUNTER += 1
  ^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
end

# Constants inside class bodies within example groups
describe SomeClass do
  class DummyClass
  ^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub class constant instead of declaring explicitly.
    INNER_CONST = "leaky"
    ^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
  end
end

# Constants inside module bodies within example groups
describe SomeClass do
  module DummyModule
  ^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub module constant instead of declaring explicitly.
    MODULE_CONST = "leaky"
    ^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeakyConstantDeclaration: Stub constant instead of declaring explicitly.
  end
end
