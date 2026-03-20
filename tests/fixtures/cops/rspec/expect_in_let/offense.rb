let(:foo) do
  expect(something).to eq 'foo'
  ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
end
let(:bar) do
  is_expected.to eq 'bar'
  ^^^^^^^^^^^ RSpec/ExpectInLet: Do not use `is_expected` in let
end
let(:baz) do
  expect_any_instance_of(Something).to receive :foo
  ^^^^^^^^^^^^^^^^^^^^^^ RSpec/ExpectInLet: Do not use `expect_any_instance_of` in let
end
let(:nested_block) do
  items.each { |i| expect(i).to be_valid }
                   ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
end
let(:conditional) do
  if condition
    expect(value).to eq(1)
    ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
  end
end
let(:ternary) do
  condition ? expect(value).to(eq(1)) : nil
              ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
end
let(:logical_and) do
  valid && expect(result).to(be_truthy)
           ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
end
let(:rescue_block) do
  begin
    expect(something).to eq(1)
    ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
  rescue StandardError
    nil
  end
end
let(:nested_expect) do
  expect do
  ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
    expect do
    ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
      DummiesIndex.bulk body: [{index: {_id: 42}}]
    end.not_to update_index(DummiesIndex)
  end
end
let :symbol_syntax do
  expect(value).to eq(1)
  ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
end
let :rescue_symbol do
  begin
    fail "something went wrong"
  rescue => error
    expect(error).to receive(:backtrace).and_return([])
    ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
    error
  end
end
let(:with_def) do
  Class.new(Base) do
    include RSpec::Matchers
    def visit_me
      expect(location).to eq '/visit_me'
      ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
    end
  end
end
let!(:with_def_bang) do
  Class.new(Base) do
    include RSpec::Matchers
    def check_value
      expect(self).to respond_to :something
      ^^^^^^ RSpec/ExpectInLet: Do not use `expect` in let
    end
  end
end
