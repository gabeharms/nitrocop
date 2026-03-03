case foo
when 1
  do_something
when 2
  do_other
end
case bar
when :a
  handle_a
end
# Empty when with comment — not flagged when AllowComments is true (default)
case storage
when :s3
  process_s3
when :fog, :azure
  # Not supported
when :filesystem
  process_fs
end
# Inline comment on when line (AllowComments: true by default)
case line
when /^\s+not a dynamic executable$/ # ignore non-executable files
when :other
  handle(line)
end
case char
when 'C' ; # ignore right key
when 'D' ; # ignore left key
else
  handle(char)
end
case value
when 2 then # comment
when 3
  do_something
end
