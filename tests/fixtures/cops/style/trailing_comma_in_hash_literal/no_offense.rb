{a: 1, b: 2}

{a: 1}

{}

{x: "hello", y: "world"}

{foo: :bar}

# Comma inside a comment between last element and closing brace
{
  'name' => 'hello'
  # No language, should default
}

# Heredoc value whose body contains a comma — not a trailing comma
{
  key: <<RUBY
hello, world
RUBY
}

# Heredoc in array value with comma in body
{
  key: [method(<<RUBY)]
foo(a, b)
RUBY
}

# Heredoc with delete(',') pattern — comma in string, not trailing
{
  foo: 'foo',
  bar: 'bar'.delete(',')
}

# nitrocop-config: Style/TrailingCommaInHashLiteral: { EnforcedStyleForMultiline: comma }
# Heredoc with trailing comma on heredoc opening line (not on closing bracket line)
{
  type: "thinking",
  thinking: <<~TEXT,
    some content
  TEXT
}
