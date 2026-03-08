it do
  foo = instance_double("Foo")
end
it do
  foo = class_double("Foo")
end
it do
  foo = object_double("Foo")
end
it do
  foo = double
end
it do
  foo = double(call: :bar)
end
it do
  # Explicit hash arg is method stubs, not a name.
  foo = double({ body: { id: '1234' } })
end
