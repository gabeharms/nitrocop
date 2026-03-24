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

# Multi-line if with comment containing semicolon after condition (FP fix)
if (spec_override = status["replicas"].presence) # ignores possibility of surge; need a spec_replicas arg for that
  result["spec"]["replicas"] = spec_override
end

# Simple if with comment containing semicolon
if condition # this is a comment; with semicolon
  do_something
end

# Unless with comment containing semicolon
unless done # not done; keep going
  process
end

# Nested if with semicolon inside parent if with semicolon (RuboCop ignore_node)
# The outer if is flagged, but the inner if is suppressed via part_of_ignored_node?
if is_real?;
  if @re>=0; return foo
  else return bar
  end
end

# Nested if with semicolon inside elsif with semicolon
if other.kind_of?(Quaternion); ((self.log)*other).exp
elsif other.kind_of?(Integer);
  if other==0; return One
  elsif other>0; x = self
  end
end
