RSpec.describe User do
  let(:params) { foo }

  subject { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `let` declarations.
end

RSpec.describe Post do
  before { setup }

  subject { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `before` declarations.
end

RSpec.describe Comment do
  it { is_expected.to be_present }

  subject { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `it` declarations.
end

shared_examples 'sortable' do
  let(:records) { create_list(:record, 3) }

  subject { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `let` declarations.
end

shared_context 'with authentication' do
  before { sign_in(user) }

  subject { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `before` declarations.
end

RSpec.describe User do
  describe '#valid?' do
    let(:params) { foo }

    subject { described_class.new }
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `let` declarations.
  end
end

RSpec.describe User do
  shared_examples_for 'a model' do
    it { is_expected.to be_present }
  end

  subject { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `shared_examples_for` declarations.
end

RSpec.describe User do
  let!(:record) { create(:record) }

  subject! { described_class.new }
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `let!` declarations.
end

RSpec.describe User do
  it_behaves_like 'sortable' do
    let(:params) { foo }

    subject { described_class.new }
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `let` declarations.
  end
end

RSpec.describe User do
  include_context 'with setup' do
    before { setup }

    subject { described_class.new }
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `before` declarations.
  end
end
