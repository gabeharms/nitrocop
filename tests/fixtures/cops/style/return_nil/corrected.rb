def foo
  return
end

def bar
  return if something
end

def baz
  return unless condition
end

# lambda do...end creates its own scope — return nil IS flagged
parse = lambda do |field|
  return
end

# lambda do...end nested inside an outer iterator block — still flagged
items.each do |item|
  handler = lambda do |model|
    return unless model.respond_to?(:model_name)
  end
end
