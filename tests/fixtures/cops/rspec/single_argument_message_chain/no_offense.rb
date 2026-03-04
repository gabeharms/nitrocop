before do
  allow(foo).to receive_message_chain(:one, :two) { :three }
end

before do
  allow(foo).to receive_message_chain("one.two") { :three }
end

before do
  foo.stub_chain(:one, :two) { :three }
end

before do
  allow(foo).to receive(:one) { :two }
end

before do
  allow(controller).to receive_message_chain "forum.moderator?" => false
end

before do
  allow(controller).to stub_chain "admin?" => true
end
