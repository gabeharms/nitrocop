foo do |x|
 x 
end

bar do
 puts 'hello' 
end

baz do |a, b|
 a + b 
end

foo do
 
end

foo do
 bar(_1) 
end

->(arg) do
 foo arg 
end

lambda do
 |arg| foo(arg) 
end
