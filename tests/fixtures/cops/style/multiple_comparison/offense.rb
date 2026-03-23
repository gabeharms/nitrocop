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
