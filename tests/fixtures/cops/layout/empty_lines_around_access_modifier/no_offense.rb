class Foo
  def bar
  end

  private

  def baz
  end

  protected

  def qux
  end
end

# Access modifier right after class opening (no blank needed before)
class Bar
  private

  def secret
  end
end

# Access modifier right before end (no blank needed after)
class Baz
  def stuff
  end

  private
end

# Comment before modifier counts as separator
class Qux
  def stuff
  end

  # These methods are private
  private

  def secret
  end
end

# Access modifier as first statement in a block body (no blank needed before)
Class.new do
  private

  def secret
  end
end

# Struct with block
Struct.new("Post", :title) do
  private

  def secret
  end
end

# `private` used as a hash value (not an access modifier)
class Message
  def webhook_data
    {
      message_type: message_type,
      private: private,
      sender: sender
    }
  end

  private

  def secret
  end
end

# `private` used inside a method body conditional (not an access modifier)
class Conversation
  def update_status
    if waiting_present && !private
      clear_waiting
    end
  end

  private

  def clear_waiting
  end
end

# Access modifiers inside a block body are checked like class bodies
app = Sinatra.new do
  private

  def priv; end

  public

  def pub; end
end

# `private` inside a method body (not a bare access modifier)
class Worker
  def private?
    private
  end
end

# `private` deep inside a method body conditional
class Processor
  def private?
    if true
      private
    end
  end
end

# Multiline class definition with superclass on next line
class Controller <
      Base
  private

  def action
  end
end

# Multiline class with sclass on next line
class <<
      self
  private

  def action
  end
end

# Access modifier with trailing comment (blank lines present)
class Config
  def setup
  end

  private # internal helpers

  def helper
  end
end

# Access modifier inside a block at the beginning (no blank needed before)
included do
  private

  def test; end
end

# Access modifier inside a block with arguments
included do |foo|
  private

  def test; end
end

# Access modifier inside a brace block at the beginning
included {
  private

  def test; end
}

# Access modifier inside a nested block
ActiveSupport.on_load(:active_storage_attachment) do
  private

  def secret
  end
end

# Nested DSL block inside another block should not be treated as a visibility scope
it "builds a Sinatra app" do
  app = Sinatra.new do
    private
    def priv; end
    public
    def pub; end
  end
end

# A comment before a top-level access modifier counts as a separator
# comment
private

def top_level_helper
end

# `public` used as an RSpec let variable receiver — not an access modifier
context "user not signed in" do
  context "given a public post" do
    let(:public) { post(:status_message, text: "hello", public: true) }

    it "shows a public post" do
      get :show, params: {id: public.id}
    end
  end
end

# `public` used as hash value in argument list — not an access modifier
describe "messages" do
  let(:public) { recipient.nil? }
  let(:local_parent) { create(:status_message, author: person, public: public) }
  let(:remote_parent) { create(:status_message, author: other.person, public: public) }
end

# `private` inside a single-line inline class — same line as sibling
it "does not delegate private methods" do
  object = Class.new{ private; def hello_world; end }.new
end

# `private` inside single-line class_eval — same line as sibling
it "returns true for own private methods" do
  Decorator.class_eval{private; def hello_world; end}
end
