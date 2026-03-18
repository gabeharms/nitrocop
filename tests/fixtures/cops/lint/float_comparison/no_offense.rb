x == 0.0
x != 0.0
x == 1
x == nil
x == y
(x - 0.1).abs < Float::EPSILON
x.to_f == 0
x.to_f == 0.0
x.to_f == nil

case value
when 0.0
  foo
end

case value
when 1
  foo
when 'string'
  bar
end

x == 1.1.ceil
x == 1.1.floor
x == 1 + 2
x == (1)
# Parenthesized float receiver with instance method - receiver.float_type? is false for (0.0)
(-0.0).next_float == (0.0).next_float
(0.0).prev_float == (-0.0).prev_float
# round with positive precision on non-literal float receiver
config.to_f.round(1) == target.to_f.round(1)
