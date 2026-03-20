create_list :user, 3
create_list(:user, 5, :trait)
1.times { create :user }
3.times { |n| create :user, position: n }
3.times { do_something }
3.times {}
3.times { |n| create :user, repositories_count: rand }
# Array with interpolated symbol factory names (not identical)
%w[fandom character].each do |type|
  [create(:"canonical_#{type}"), create(:"canonical_#{type}")]
end
# Array with different create calls
[create(:user), create(:user, age: 18)]
# Array with single create
[create(:user)]
# Empty array
[]
# Array.new with create containing array args with method calls
records = Array.new(3) { FactoryBot.create(:record, :tag_ids => [@tag.id]) }
items = Array.new(5) { create(:item, names: [user.name]) }
# Value omission without local variable: Prism wraps CallNode inside
# ImplicitNode (Parser sees `(send nil :customer)`), so it's a method call.
# RuboCop skips these via arguments_include_method_call?.
3.times { create(:subscription, customer:) }
3.times { create(:item, checklist:, checked: true) }
2.times { create(:refund, purchase:, amount_cents: 10) }
2.times.map do
  create(:role_appointment, person:)
end
# Array with create calls where one has a block and the other does not
[create(:invoice, organization:), create(:invoice, organization:) { |i| create(:metadata, invoice: i) }]
