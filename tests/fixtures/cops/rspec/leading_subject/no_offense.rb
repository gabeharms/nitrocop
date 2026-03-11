RSpec.describe User do
  subject { described_class.new }

  let(:params) { foo }

  context 'nested' do
    subject { described_class.new }
    it { is_expected.to be_valid }
  end
end

RSpec.describe Post do
  subject { described_class.new }

  before { setup }
  it { is_expected.to be_present }
end

module Spree
  describe LegacyUser do
    let(:user) { create(:user) }
    before { setup }
    subject { described_class.new }
  end
end

require 'spec_helper'
module Berkshelf
  describe ChefRepoUniverse do
    let(:fixture) { nil }
    subject { described_class.new(fixture) }
  end
end

class Configuration
  describe Server do
    let(:server) { build(:server) }
    subject { described_class.new }
  end
end

shared_examples 'sortable' do
  subject { described_class.new }
  let(:records) { create_list(:record, 3) }
end

shared_context 'with authentication' do
  subject { described_class.new }
  before { sign_in(user) }
end

RSpec.describe User do
  describe '#valid?' do
    subject { described_class.new }
    let(:params) { foo }
  end
end

RSpec.describe User do
  let(:foo) { 'bar' }

  it_behaves_like 'a good citizen' do
    subject { described_class.new }
  end
end

RSpec.describe User do
  it "doesn't mind me calling a method called subject in the test" do
    let(foo)
    subject { bar }
  end
end
