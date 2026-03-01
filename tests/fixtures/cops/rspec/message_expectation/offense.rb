expect(foo).to receive(:bar)
^^^^^^ RSpec/MessageExpectation: Prefer `allow` for setting message expectations.
expect(foo).to receive(:baz).with(1)
^^^^^^ RSpec/MessageExpectation: Prefer `allow` for setting message expectations.
expect(obj).to receive(:qux).and_return(true).at_least(:once)
^^^^^^ RSpec/MessageExpectation: Prefer `allow` for setting message expectations.
