if condition
  do_x
  do_z
  ^^^^ Style/IdenticalConditionalBranches: Move `do_z` out of the conditional.
else
  do_y
  do_z
  ^^^^ Style/IdenticalConditionalBranches: Move `do_z` out of the conditional.
end

if foo
  bar
  result
  ^^^^^^ Style/IdenticalConditionalBranches: Move `result` out of the conditional.
else
  baz
  result
end
