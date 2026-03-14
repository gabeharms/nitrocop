create(:user)
build(:user)
build_list(:user, 10)
create_list(:user, 10)
build_stubbed(:user)
build_stubbed_list(:user, 10)
create
create foo: :bar

# FactoryBot call used as argument to another method (ambiguous without parens)
sign_in create :user
sign_in create :moderator
described_class.add attributes_for :item, name: 'attribute'

# Factory call as sole statement in if body (ambiguous in Parser: parent is :if)
if condition
  create :item, name: 'test'
end

# Factory call as sole statement in else body (ambiguous in Parser: parent is :if)
if condition
  something_else
else
  create :organization
end
