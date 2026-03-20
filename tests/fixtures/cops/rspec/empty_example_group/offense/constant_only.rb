describe Foo do
  describe '#to_type' do
  ^^^^^^^^^^^^^^^^^^^^ RSpec/EmptyExampleGroup: Empty example group detected.
    FORMATS = {
      'General' => :float,
      '0' => :float,
    }.each do |format, type|
      it "translates #{format} to #{type}" do
        expect(described_class.to_type(format)).to eq(type)
      end
    end
  end
end
