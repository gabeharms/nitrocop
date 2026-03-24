class Foo
  def bar
    <<~SQL
      'Hi'
  SQL
  ^^^ Layout/ClosingHeredocIndentation: `SQL` is not aligned with `<<~SQL`.
  end
end

class Baz
  def qux
    <<~RUBY
      something
        RUBY
        ^^^^ Layout/ClosingHeredocIndentation: `RUBY` is not aligned with `<<~RUBY`.
  end
end

def example
  <<-TEXT
    hello
      TEXT
      ^^^^ Layout/ClosingHeredocIndentation: `TEXT` is not aligned with `<<-TEXT`.
end

# Heredoc in block body should still flag offense
get '/foo' do
    <<-EOHTML
    <html></html>
EOHTML
^^^^^^ Layout/ClosingHeredocIndentation: `EOHTML` is not aligned with `<<-EOHTML`.
end

# Heredoc as argument with wrong closing alignment (matches neither opening nor call)
include_examples :offense,
                 <<-HEREDOC
  bar
    HEREDOC
    ^^^^^^^ Layout/ClosingHeredocIndentation: `HEREDOC` is not aligned with `<<-HEREDOC` or beginning of method definition.
