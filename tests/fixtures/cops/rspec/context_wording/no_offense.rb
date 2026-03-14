context 'when the display name is not present' do
end

context 'with valid input' do
end

context 'without any errors' do
end

describe 'the display name not present' do
end

# Calls with receiver should not be flagged
obj.context 'the display name not present' do
end

# Calls without a block should not be flagged
context 'some non-prefix text'

# Block-argument style (proc passed as &arg) should not be flagged
# RuboCop's on_block only fires for BlockNode, not BlockArgumentNode (&proc)
context 'Cache', if: condition, &(proc do
end)

# Prefix followed by non-word characters (hyphen, dot, colon) should not be flagged
# RuboCop uses \b word boundary which matches at non-word chars
context 'when-something-happens' do
end

context 'with.dots.in.name' do
end

context 'without:colons' do
end
