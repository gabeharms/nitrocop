if foo
  bar
end

foo ? bar : baz

bar if foo

if foo
  bar
else
  baz
end

# Multi-line if with comment containing semicolons should not be flagged
if quantifier1 == quantifier2
  # (?:a+)+ equals (?:a+) ; (?:a*)* equals (?:a*)
  quantifier1
else
  '*'
end

# Multi-line if with semicolons in comment between condition and body
if provider == 'whatsapp_cloud'
  # The callback is for manual setup flow; embedded signup handles it
  default_config = {}
end

# else if pattern: inner if has parent that is if_type, RuboCop skips it
if x > 0
  foo
else if y > 0; bar else baz end
end

# Nested if inside else branch (parent is if_type)
if a
  something
else if b; c end
end
