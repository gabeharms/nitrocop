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
