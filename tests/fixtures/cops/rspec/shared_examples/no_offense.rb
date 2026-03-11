it_behaves_like 'foo bar baz'
shared_examples 'some thing' do
  it 'works' do
    expect(true).to be true
  end
end
include_examples 'hello world'
shared_examples_for 'another example' do
  it 'passes' do
    expect(1).to eq(1)
  end
end
shared_context 'my context' do
  let(:value) { 42 }
end
include_context 'test setup'
RSpec.shared_context 'rspec context' do
  let(:name) { 'test' }
end
