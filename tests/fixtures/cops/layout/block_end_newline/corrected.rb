blah do |i|
  foo(i)
end
blah { |i|
  foo(i)
}
items.each do |x|
  bar(x)
end
-> do
  foo
end

-> {
  foo
}

-> do
  foo
end

foo { |
;x|
}
