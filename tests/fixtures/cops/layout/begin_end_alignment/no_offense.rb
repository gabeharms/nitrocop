x = begin
  1
end

y = begin
  2
end

begin
  foo
end

def method_body
  bar
end

result ||= begin
  compute
end

# Tab-indented begin/end (should not flag)
if outfile
	begin
	end
end

def tab_method
	x = begin
		1
	end
end
