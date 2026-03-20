it 'does something' do
  skip 'not yet implemented' do
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SkipBlockInsideExample: Don't pass a block to `skip` inside examples.
  end
end
specify 'another test' do
  skip 'wip' do
  ^^^^^^^^^^^^^^ RSpec/SkipBlockInsideExample: Don't pass a block to `skip` inside examples.
  end
end
it 'third example' do
  skip 'todo' do
  ^^^^^^^^^^^^^^^ RSpec/SkipBlockInsideExample: Don't pass a block to `skip` inside examples.
    some_code
  end
end
it "with skip nested deeper" do
  with_new_environment do
    RSpec.describe "some skipped test" do
      skip "foo" do
      ^^^^^^^^^^^^^^ RSpec/SkipBlockInsideExample: Don't pass a block to `skip` inside examples.
        expect(1 + 1).to eq(5)
      end
    end
  end
end
it 'skip with block arg inside nested blocks' do
  database 'Customer' do
    table 'users' do
      skip { |index, record| record['age'] > 18 }
      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SkipBlockInsideExample: Don't pass a block to `skip` inside examples.
    end
  end
end
