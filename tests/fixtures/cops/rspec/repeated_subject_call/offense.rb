RSpec.describe Foo do
  it do
    subject
    expect { subject }.to not_change { Foo.count }
             ^^^^^^^ RSpec/RepeatedSubjectCall: Calls to subject are memoized, this block is misleading
  end
end

RSpec.describe Bar do
  it do
    expect { subject }.to change { Bar.count }
    expect { subject }.to not_change { Bar.count }
             ^^^^^^^ RSpec/RepeatedSubjectCall: Calls to subject are memoized, this block is misleading
  end
end

RSpec.describe Baz do
  it do
    subject
    nested_block do
      expect { on_shard(:europe) { subject } }.to not_change { Baz.count }
                                   ^^^^^^^ RSpec/RepeatedSubjectCall: Calls to subject are memoized, this block is misleading
    end
  end
end

# Named subject alias
RSpec.describe Qux do
  subject(:bar) { do_something }

  it do
    bar
    expect { bar }.to not_change { Qux.count }
             ^^^ RSpec/RepeatedSubjectCall: Calls to subject are memoized, this block is misleading
  end
end

# Named subject used as constant path parent (mod::Params)
RSpec.describe TypeModule do
  subject(:mod) { Dry::Types.module }

  it "adds strict types as default" do
    expect(mod::Integer).to be(Dry::Types["integer"])
    expect(mod::Nominal::Integer).to be(Dry::Types["nominal.integer"])
    expect { mod::Params }.to raise_error(NameError)
             ^^^ RSpec/RepeatedSubjectCall: Calls to subject are memoized, this block is misleading
  end
end
