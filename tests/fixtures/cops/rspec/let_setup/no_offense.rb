describe Foo do
  let!(:foo) { bar }

  before do
    foo
  end

  it 'does not use foo' do
    expect(baz).to eq(qux)
  end
end

describe Foo do
  let!(:foo) { bar }

  it 'uses foo' do
    foo
    expect(baz).to eq(qux)
  end
end

# let! name referenced in a sibling let! body — should not be flagged
describe Widget do
  let!(:user) { create(:user) }
  let!(:post) { create(:post, author: user) }

  it 'creates a post' do
    expect(post).to be_valid
  end
end

# let! that overrides an outer let! — should not be flagged
describe Widget do
  let!(:record) { create(:widget) }

  it 'uses record' do
    expect(record).to be_valid
  end

  context 'when record is nil' do
    let!(:record) { nil }

    it 'handles nil' do
      expect(true).to be true
    end
  end

  context 'when record is special' do
    let!(:record) { create(:widget, special: true) }

    it 'handles special' do
      expect(true).to be true
    end
  end
end

# let! overriding outer let! in deeply nested context
describe Service do
  let!(:user) { create(:user) }

  it 'allows access' do
    expect(user).to be_valid
  end

  context 'when user is admin' do
    context 'and user is blocked' do
      let!(:user) { create(:user, :blocked) }

      it 'denies access' do
        expect(true).to be true
      end
    end
  end
end
