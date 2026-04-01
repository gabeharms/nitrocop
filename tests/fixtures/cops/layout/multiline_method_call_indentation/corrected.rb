foo
  .bar
  .baz

thing
  .first
  .second
  .third

query
  .select('foo')
  .where(x: 1)
  .order(:name)

# Block chain continuation: .sort_by should align with .with_index dot
frequencies.map.with_index { |f, i| [f / total, hex[i]] }
               .sort_by { |r| -r[0] }

# Multiline receiver chain with single-line block: .sort_by should align with .with_index dot
submission.template_submitters
          .group_by.with_index { |s, index| s['order'] || index }
                   .sort_by(&:first).pluck(1)

# Hash pair value: chain should align with chain root start column
foo(key: receiver.chained
         .misaligned)

bar = Foo
      .a
      .b(c)

# Trailing dot: unaligned methods (aligned style)
User.a
    .b
    .c

# Trailing dot: misaligned in assignment
a = b.c.
    d

# Unaligned method in block body
a do
  b.c
   .d
end

# Hash pair value: misaligned multi-dot chain
method(key: value.foo.bar
            .baz)
