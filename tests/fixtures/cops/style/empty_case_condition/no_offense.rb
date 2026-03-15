case :a
when 1 == 2
  foo
when 1 == 1
  bar
else
  baz
end

case x
when 1
  foo
end

# send parent: case used as method receiver
case
when true, false; 'foo'
end.should == 'foo'

# send parent: case passed as method argument
do_something case
             when foo
               1
             else
               2
             end

# return parent
return case
       when foo
         1
       else
         2
       end

# break parent
break case
      when foo
        1
      else
        2
      end

# next parent
next case
     when foo
       1
     else
       2
     end

# branches contain return statements
case
when cond_a
  return compile_plain(node)
when cond_b
  process(node)
end

# branches contain return in descendant
case
when foo
  if bar
    return 1
  end
  2
else
  3
end
