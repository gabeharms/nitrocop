describe Foo do
  subject(:foo) { described_class.new }

  before do
    allow(foo).to receive(:bar).and_return(baz)
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SubjectStub: Do not stub methods of the object under test.
  end

  it 'uses expect twice' do
    expect(foo.bar).to eq(baz)
  end
end

describe Bar do
  subject(:bar) { described_class.new }

  before do
    expect(bar).to receive(:baz)
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SubjectStub: Do not stub methods of the object under test.
  end

  it 'tests bar' do
    expect(bar.baz).to eq(true)
  end
end

describe Baz do
  subject { described_class.new }

  it 'stubs subject' do
    expect(subject).to receive(:qux)
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SubjectStub: Do not stub methods of the object under test.
  end
end

# do...end block on receive chain followed by chain method
describe Processor do
  subject { described_class.new }

  it 'detects do...end with chain' do
    expect(subject).to receive(:process).and_wrap_original do |original, item|
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SubjectStub: Do not stub methods of the object under test.
      original.call(item)
    end.at_least(:once)
    subject.call
  end
end

# Explicit parens on .to(receive(...)).and_return(...)
describe Handler do
  subject { described_class.new }

  it 'detects explicit parens with chain' do
    allow(subject).to(receive(:load_resource)).and_return(resource)
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SubjectStub: Do not stub methods of the object under test.
    subject.run
  end
end

# do...end block on receive chain followed by .and_return
describe Forwarder do
  subject(:forwarder) { described_class.new }

  it 'detects do...end with and_return' do
    expect(forwarder).to receive(:spawn) do |*args|
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/SubjectStub: Do not stub methods of the object under test.
      expect(args).to start_with('ssh')
    end.and_return(9999)
    forwarder.forward
  end
end
