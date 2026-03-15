def foo
  puts "hello"
end

# Regular comment
x = 1
y = 2
z = x + y

# Directives inside string literals should not be flagged
code = <<~RUBY
  # rubocop:disable Metrics/MethodLength
  def long_method
  end
  # rubocop:enable Metrics/MethodLength
RUBY

template = "# rubocop:disable Style/FrozenStringLiteralComment"

multiline = <<-HEREDOC
  # rubocop:todo Layout/LineLength
  some_long_line
HEREDOC
