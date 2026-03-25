it 'does something' do
  expect(true).to eq(true)
end
it 'returns the correct value' do
  expect(1 + 1).to eq(2)
end
specify 'works correctly' do
  expect(subject).to be_valid
end
it 'is valid' do
  expect(user).to be_valid
end
it 'displays shoulder text' do
  expect(page).to have_content('shoulder')
end
# specify is not checked (RuboCop only checks `it` blocks)
specify 'should do something' do
  expect(true).to be true
end
# Pending examples (no block) — RuboCop skips these
it "should limit owners to only updating owner-accessible fields"
it "should limit admins to only updating admin-accessible fields"
it "should limit members to only updating member-accessible fields"
# fit/xit are not checked — RuboCop only matches :it method
fit 'should do something' do
  expect(true).to be true
end
xit 'should be valid' do
  expect(subject).to be_valid
end
# "shouldnt" without apostrophe — no word boundary after "should"
it 'shouldnt create a record' do
  expect(Record.count).to eq(0)
end
it 'shouldnt collide with other data' do
  expect(true).to eq(true)
end
# &(proc do...end) — Prism binds do...end to outer `it`, but
# RuboCop's Parser gem binds it to `proc`, so no offense fires
it 'should convert the example', &(proc do
  expect(true).to eq(true)
end)
# Double-quoted strings with non-UTF-8 hex escapes — RuboCop skips these
it "should convert emoticon &#xe63e; to \xf8\x9f in B-Encoding" do
end
it "should convert emoticon &#xe63e; to \xf8\x9f" do
end
it "should convert emoticon &#xe63e; to \xf9\x8b in B-Encoding" do
end
it "should convert emoticon &#xe63e; to \xf9\x8b" do
end
