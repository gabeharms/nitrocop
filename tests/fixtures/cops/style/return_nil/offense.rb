def foo
  return nil
  ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
end

def bar
  return nil if something
  ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
end

def baz
  return nil unless condition
  ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
end

# lambda do...end creates its own scope — return nil IS flagged
parse = lambda do |field|
  return nil
  ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
end

# lambda do...end nested inside an outer iterator block — still flagged
items.each do |item|
  handler = lambda do |model|
    return nil unless model.respond_to?(:model_name)
    ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
  end
end

# return nil inside bare proc (no receiver) IS flagged — proc without
# receiver is not a chained send, so RuboCop does not suppress.
def method_with_proc
  handler = proc do |result|
    return nil if result.nil?
    ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
  end
end

# proc inside hash value inside method call — still bare proc, still flagged
def method_with_proc_in_hash
  SomeApi.run(
    handlers: {
      '*' => proc do |result|
        log("error: #{result[:status]}")
        return nil
        ^^^^^^^^^^ Style/ReturnNil: Use `return` instead of `return nil`.
      end
    }
  )
end
