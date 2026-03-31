Const.do_something

self.foo

foo.to_s.strip

42.minutes

'hello'.upcase

foo.to_h

foo.to_a

foo.to_i

foo.to_f

foo.to_s

foo.to_h { |k, v| [k, v] }

# Case 5: AllowedMethods in conditional context
if foo.respond_to?(:bar)
  do_something
elsif foo.respond_to?(:baz)
  do_something_else
end

do_something unless foo.respond_to?(:bar)

while foo.respond_to?(:bar)
  do_something
end

begin
  do_something
end until foo.respond_to?(:bar)

do_something if foo.respond_to?(:bar) && !foo.respond_to?(:baz)

if foo.is_a?(String)
  do_something
end

do_something if foo.kind_of?(Hash)

return unless foo.eql?('bar')

foo.instance_of?(String) ? 'yes' : 'no'

# AllowedMethods with || in condition
return unless options[:name] && options[:value].is_a?(Hash)

# AllowedMethods with eql? in if condition
if parameters[:method].eql?('POST')
  handle_post
end

# equal? in condition
do_something if foo.equal?(bar)

# AllowedMethods in standalone && / || expressions (not inside if/unless)
user.is_a?(Admin) && user.respond_to?(:roles)

options[:min].is_a?(Proc) && options[:max].is_a?(Proc)

condition.respond_to?(:method) || fallback

# Backtick literal receiver — always returns String, &. is redundant
`cat /tmp/pid`.strip

# Negation with ! wrapping AllowedMethod in || (standalone)
!charge.is_a?(Klass) || fallback

# respond_to? with string argument (not symbol) in &&
obj.respond_to?('some_method?') && obj.some_method?

# Negation with ! in standalone || expression
!item.is_a?(Widget) || !(item&.active? || item&.pending?)

# AllowedMethod in ternary condition (via &&)
obj.is_a?(String) && obj.valid? ? 'yes' : 'no'

# Chained calls ending with &.is_a? in || false
node.args&.last.is_a?(TrueNode) || false

# AllowedMethod in && inside parentheses inside ||
node.nil? || (node&.name&.to_s == "foo" && node.parent.is_a?(SelfNode))

# if with parenthesized && inside condition
if persisted? && active? && (!obj.is_a?(Klass) || !(obj&.running?))
  do_something
end

# Assignment with ||= and parenthesized &&
zone ||= (user.is_a?(Admin) && user.valid?) ? user.zone : default

# AllowedMethod in && inside method body
def check_roles?(roles)
  user.is_a?(Admin) && user.respond_to?(:roles)
end

# rescue => self&.foo — self is never nil, &. is redundant
begin
  something
rescue => self.captured_error
  handle
end
