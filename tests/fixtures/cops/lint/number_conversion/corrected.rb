Integer('10', 10)
Float('10.2')
Rational('1/3')
# Safe navigation should still be flagged
Integer("10", 10)
# Symbol form: map(&:to_i)
"1,2,3".split(',').map(&:to_i)
# Symbol form: try(:to_f)
"foo".try(:to_f)
# Symbol form: send(:to_c)
"foo".send(:to_c)
# Symbol form without parentheses
"1,2,3".split(',').map &:to_i
# Symbol form with safe navigation
"1,2,3".split(',')&.map(&:to_i)
# Bare symbol form without explicit receiver (implicit self)
map(&:to_i)
try(:to_f)
send(:to_c)
# Qualified constant (Core::Utils::Time) does NOT match "Time" in IgnoredClasses
Integer(Core::Utils::Time.now, 10)
Integer(Faker::Time.backward(days: 365), 10)
# Symbol argument with regular block (not block argument) should still be flagged
receive(:to_i) { 1 }
