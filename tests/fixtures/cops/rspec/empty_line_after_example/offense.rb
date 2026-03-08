RSpec.describe Foo do
  it 'does this' do
  end
  ^^^ RSpec/EmptyLineAfterExample: Add an empty line after `it`.
  it 'does that' do
  end

  specify do
  end
  ^^^ RSpec/EmptyLineAfterExample: Add an empty line after `specify`.
  specify 'something else' do
  end

  it 'another example' do
  end
  ^^^ RSpec/EmptyLineAfterExample: Add an empty line after `it`.
  # a comment
  it 'yet another' do
  end

  # One-liner followed by multi-liner should fire
  it("returns false") { expect(true).to be false }
  ^^ RSpec/EmptyLineAfterExample: Add an empty line after `it`.
  it "adds the errors" do
    expect(true).to be true
  end

  # Description text containing `end` should not be mistaken for one-line do..end
  it("returns the number of added lines") { is_expected.to eq(1) }
  ^^ RSpec/EmptyLineAfterExample: Add an empty line after `it`.
  it "adds a line to the end" do
    expect(true).to be true
  end
end
