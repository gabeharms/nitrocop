blah do |i|
  foo(i) end
         ^^^ Layout/BlockEndNewline: Expression at 2, 10 should be on its own line.
blah { |i|
  foo(i) }
         ^ Layout/BlockEndNewline: Expression at 4, 10 should be on its own line.
items.each do |x|
  bar(x) end
         ^^^ Layout/BlockEndNewline: Expression at 6, 10 should be on its own line.
-> do
  foo
end

-> {
  foo }
      ^ Layout/BlockEndNewline: Expression at 12, 7 should be on its own line.

-> do
  foo end
      ^^^ Layout/BlockEndNewline: Expression at 15, 7 should be on its own line.
