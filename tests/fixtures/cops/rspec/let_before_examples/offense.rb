RSpec.describe User do
  it { is_expected.to be_valid }
  let(:foo) { bar }
  ^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let` before the examples in the group.
end

RSpec.describe Post do
  context 'nested' do
    it { is_expected.to be_valid }
  end
  let(:baz) { qux }
  ^^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let` before the examples in the group.
end

RSpec.describe Comment do
  it_behaves_like 'shared stuff'
  let!(:user) { create(:user) }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let!` before the examples in the group.
end

# RSpec.feature should also be recognized as an example group
RSpec.feature SearchController, type: :controller do
  let!(:record_without_links) { create :record }

  it "returns nothing when parent has no children" do
    expect(results.count).to eq 0
  end

  let!(:three_links) { create_list :link, 3 }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let!` before the examples in the group.
  let!(:record_with_three_links) { create :record, links: three_links }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let!` before the examples in the group.

  it "returns three records" do
    expect(results.count).to eq 3
  end

  let!(:five_links) { create_list :link, 5 }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let!` before the examples in the group.
  let!(:record_with_five_links) { create :record, links: five_links }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LetBeforeExamples: Move `let!` before the examples in the group.

  it "returns five records" do
    expect(results.count).to eq 5
  end
end
