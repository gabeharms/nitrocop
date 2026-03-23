a = "test"
a == "x" || a == "y"
^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

a == "x" || a == "y" || a == "z"
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

"x" == a || "y" == a
^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# With method call comparisons mixed in (method calls are skipped but non-method comparisons count)
a == foo.bar || a == 'x' || a == 'y'
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# Method call as the "variable" being compared
name == :invalid_client || name == :unauthorized_client
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

foo.bar == "a" || foo.bar == "b"
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# Nested inside && within a larger || — each parenthesized || group is independent
if (height > width && (rotation == 0 || rotation == 180)) || (height < width && (rotation == 90 || rotation == 270))
                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.
                                                                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.
end

# Method call as variable compared against symbol literals
x.flag == :> || x.flag == :>=
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# Hash-like access as variable compared against local variables
outer_left_x = 48.24
outer_right_x = 547.04
lines.select { |it| it[:from][:x] == outer_left_x || it[:from][:x] == outer_right_x }
                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# Mixed-variable || chain: first variable has enough comparisons, then a different variable appears
next if language == '.' || language == '..' || language == 'Binary' || File.basename(language) == 'ace_modes.json'
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# Mixed-variable || chain: call as variable with different method at the end
if env['params'][@key].downcase == 'true' || env['params'][@key].downcase == 't' || env['params'][@key].to_i == 1
   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.

# Two comparisons of same variable then a different variable (2 >= threshold)
return 'trataPeticion' if three_ds_info == 'AuthenticationData' || three_ds_info == 'ChallengeResponse' || data[:sca_exemption] == 'MIT'
                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/MultipleComparison: Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.
