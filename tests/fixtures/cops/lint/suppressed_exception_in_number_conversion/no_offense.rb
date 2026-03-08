Integer(arg, exception: false)
Float(arg, exception: false)
Integer(arg) rescue :fallback
something rescue nil
x = Integer(arg)
# Rescue with non-ArgumentError/TypeError should not trigger
begin
  Rational(raw)
rescue ZeroDivisionError
  nil
end
begin
  Integer(arg)
rescue NameError
  nil
end
# begin block with else clause should not trigger
begin
  Integer(arg)
rescue
  nil
else
  42
end
# Float with too many args should not trigger
Float(arg, unexpected_arg) rescue nil
# begin block with multiple statements should not trigger
begin
  Integer(arg)
  do_something
rescue
  42
end
