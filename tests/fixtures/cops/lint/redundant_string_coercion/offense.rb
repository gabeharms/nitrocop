"result is #{foo.to_s}"
                 ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in interpolation.
"value: #{bar.to_s}"
              ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in interpolation.
"output: #{baz.to_s}"
               ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in interpolation.
puts first.to_s, second.to_s
           ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in `puts`.
                        ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in `puts`.
print first.to_s, second.to_s
            ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in `print`.
                         ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in `print`.
warn first.to_s, second.to_s
           ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in `warn`.
                        ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in `warn`.
"#{to_s}"
   ^^^^ Lint/RedundantStringCoercion: Use `self` instead of `Object#to_s` in interpolation.
puts to_s, to_s
     ^^^^ Lint/RedundantStringCoercion: Use `self` instead of `Object#to_s` in `puts`.
           ^^^^ Lint/RedundantStringCoercion: Use `self` instead of `Object#to_s` in `puts`.
print to_s, to_s
      ^^^^ Lint/RedundantStringCoercion: Use `self` instead of `Object#to_s` in `print`.
            ^^^^ Lint/RedundantStringCoercion: Use `self` instead of `Object#to_s` in `print`.
/regexp #{result.to_s}/
                 ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in interpolation.
`backticks #{result.to_s}`
                    ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in interpolation.
:"symbol #{result.to_s}"
                  ^^^^ Lint/RedundantStringCoercion: Redundant use of `Object#to_s` in interpolation.
