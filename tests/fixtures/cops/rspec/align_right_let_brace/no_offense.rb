RSpec.describe 'test' do
  let(:foo)      { a    }
  let(:hi)       { ab   }
  let(:blahblah) { abcd }

  let(:thing) { ignore_this }
  let(:other) {
    ignore_this_too
  }

  # Comments with let-like text should not be matched
  let(:x) { a }
  # let(:y) { ab }
  let(:z) { abc }

  # let with proc argument (no block) should not be matched
  let(:user, &args[:build_user])

  # Single let should not trigger offense (no group to align with)
  let(:solo) { value }

  # let-like text inside heredoc strings should not be matched
  it 'tests alignment' do
    expect_offense(<<~RUBY)
      let(:foo) { a }
      let(:bar) { abc }
    RUBY
  end
end
