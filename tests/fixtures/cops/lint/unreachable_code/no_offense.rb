def foo
  return 1
end

def bar
  raise 'error' if condition
  do_something
end

def baz
  if condition
    return 1
  end
  2
end

# fail/raise with a block is a DSL method call (e.g. FactoryBot), not Kernel#fail
FactoryBot.define do
  factory :item do
    success { true }
    fail { false }
    error { false }
  end
end
