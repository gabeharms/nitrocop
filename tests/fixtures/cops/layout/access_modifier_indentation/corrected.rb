class Foo
  private
  def bar; end
end

class Baz
  protected
  def qux; end
end

class Quux
  public
  def corge; end
end

Test = Module.new do
  private
  def grault; end
end

included do
  private
  def garply; end
end

class Shell
  private
      def read_line; end
end
