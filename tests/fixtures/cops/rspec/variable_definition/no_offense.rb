let(:user_name) { 'Adam' }
let(:email) { 'test@example.com' }
let!(:count) { 42 }
subject(:result) { described_class.new }
let(:items) { [1, 2, 3] }
let!(:record) { create(:record) }

# subject with receiver is not an RSpec call
message.subject 'testing'

# let with no arguments
let
