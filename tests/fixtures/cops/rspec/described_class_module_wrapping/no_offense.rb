RSpec.describe MyClass do
  subject { "MyClass" }
end

module MyModule
  def some_method
  end
end

describe Foo do
  it 'works' do
  end
end

# Bare describe (no RSpec. prefix) inside a module is NOT flagged by RuboCop
module Decidim::Accountability
  describe ResultCell, type: :cell do
    it 'renders something' do
    end
  end
end

module MyNamespace
  describe SomeService do
    it 'does work' do
    end
  end
end

# Module with RSpec.describe called as utility (no block) — not flagged
module Helpers
  def metadata_with(additional_metadata)
    ::RSpec.describe("example group").metadata.merge(additional_metadata)
  end
end

# Module with RSpec.describe called with block argument (not actual block) — not flagged
module CommonHelpers
  def describe_successfully(*args, &describe_body)
    RSpec.describe(*args, &describe_body)
  end
end
