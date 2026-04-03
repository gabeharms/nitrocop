case status
when 'active'
  run_active
when 'pending'
  run_pending
when 'archived'
  run_archived
else
  run_default
end

case x
when 1
when 2
  run_two
when 3
  run_three
end

case kind
when :foo
when :bar
  do_bar
when :baz
  do_baz
end
