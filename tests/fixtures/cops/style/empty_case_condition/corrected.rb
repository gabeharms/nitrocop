if 1 == 2
  foo
elsif 1 == 1
  bar
else
  baz
end

if 1 == 2
  foo
elsif 1 == 1
  bar
end

if 1 == 2
  foo
end

x = if foo.is_a?(String)
      1
    elsif foo.is_a?(Array)
      2
    else
      3
    end

@result = if cond_a then :a
          elsif cond_b then :b
          else :c
          end

impl = if obj.is_a?(Class)
         obj.new
       elsif obj.respond_to?(:call)
         obj.call
       else
         raise "unsupported"
       end
