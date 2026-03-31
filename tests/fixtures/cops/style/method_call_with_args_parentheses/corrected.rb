# require_parentheses style (default)

# Method calls with receiver and args but no parens
foo.bar(1, 2)

obj.method("arg")

x.send(:message, "data")

# Receiverless calls inside method defs are NOT macros
def foo
  test(a, b)
end

# Safe navigation operator also flags
top&.test(a, b)

# Multiline chained method calls — offense is at start of full expression
expect(described_class.new)
  .to(match_array(y))

custom_fields
  .include?(attribute)

# Receiverless call nested as argument to another call in class body
# is NOT a macro (parent in AST is send, not a wrapper)
class MyClass
  foo bar(:baz)
end

# Receiverless calls inside case/when in class body are NOT macros
# (case/when are not wrappers in RuboCop's in_macro_scope?)
class MyClass
  case type
  when :foo
    test(a, b)
  end
end

# Receiverless calls inside while/until in class body are NOT macros
class MyClass
  while running
    process_item(a)
  end
end

# Receiverless calls inside rescue in class body are NOT macros
# (rescue is not a wrapper in RuboCop's in_macro_scope?)
class MyClass
  begin
    test(a, b)
  rescue
    handle_error(a)
  end
end

# Receiverless calls inside ensure in class body are NOT macros
class MyClass
  begin
    test(a, b)
  ensure
    cleanup(a)
  end
end

# yield with args and no parens in method body
def each_item
  yield(element)
end

# yield with multiple args
def traverse(tree, &block)
  tree.each do |item|
    yield(item, tree)
  end
end
