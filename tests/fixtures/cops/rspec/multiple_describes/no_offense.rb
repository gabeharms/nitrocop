describe MyClass do
  it 'works' do
    expect(true).to eq(true)
  end
end

shared_examples_for 'behaves' do
end

shared_examples_for 'misbehaves' do
end

# Block-argument style (&proc) should not be counted as a top-level example group.
# RuboCop's on_block only fires for BlockNode, not BlockArgumentNode (&proc).
describe 'Conditional feature', if: condition, &(proc do
  it 'works' do end
end)
