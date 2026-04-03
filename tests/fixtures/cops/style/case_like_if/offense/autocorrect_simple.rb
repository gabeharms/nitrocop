if status == 'active'
^^^^^^^^^^^^^^^^^^^^ Style/CaseLikeIf: Convert `if-elsif` to `case-when`.
  run_active
elsif 'pending' == status
  run_pending
elsif status == 'archived'
  run_archived
else
  run_default
end

if x == 1
^^^^^^^^^^ Style/CaseLikeIf: Convert `if-elsif` to `case-when`.
elsif x == 2
  run_two
elsif x == 3
  run_three
end

if kind == :foo
^^^^^^^^^^^^^^^ Style/CaseLikeIf: Convert `if-elsif` to `case-when`.
elsif kind == :bar
  do_bar
elsif kind == :baz
  do_baz
end
