context 'when awesome case' do
  it { cool_predicate_method }
end

context 'when another awesome case' do
  it { another_predicate_method }
end

context 'rejected' do
  skip
end

context 'processed' do
  skip
end

describe 'doing x' do
  it { metadata_test_method }
end

describe 'doing x', :request do
  it { metadata_test_method }
end

describe 'included range' do
  before { @range = 1..99 }
  it { @range.should include 50 }
end

describe 'excluded range' do
  before { @range = 1...99 }
  it { @range.should include 50 }
end

context 'backtick a' do
  before { `echo hello` }
  it { should be_truthy }
end

context 'backtick b' do
  before { `echo world` }
  it { should be_truthy }
end

# Different arg placement in call chain — not a repeated body
context 'with one arg' do
  it { expect(cmd.curry(data).call('User')).to eql(result) }
end

context 'with two args' do
  it { expect(cmd.curry(data, 'User').call).to eql(result) }
end
