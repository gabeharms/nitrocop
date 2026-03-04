before do
  allow(Foo).to receive(:foo).and_return(baz)
  allow(Bar).to receive(:bar).and_return(bar)
  allow(Baz).to receive(:baz).and_return(foo)
end
before do
  allow(Service).to receive(:foo) { baz }
  allow(Service).to receive(:bar) { bar }
end
# String args to receive() should not trigger receive_messages
before(:each) do
  allow(self).to receive('action_name').and_return(action_name)
  allow(self).to receive('current_page?').and_return(false)
end
