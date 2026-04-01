a = "test"
["x", "y"].include?(a)

["x", "y", "z"].include?(a)

["x", "y"].include?(a)

# With method call comparisons mixed in (method calls are skipped but non-method comparisons count)
a == foo.bar || a == 'x' || a == 'y'

# Method call as the "variable" being compared
[:invalid_client, :unauthorized_client].include?(name)

["a", "b"].include?(foo.bar)

# Nested inside && within a larger || — each parenthesized || group is independent
if (height > width && ([0, 180].include?(rotation))) || (height < width && ([90, 270].include?(rotation)))
end

# Method call as variable compared against symbol literals
[:>, :>=].include?(x.flag)

# Hash-like access as variable compared against local variables
outer_left_x = 48.24
outer_right_x = 547.04
lines.select { |it| [outer_left_x, outer_right_x].include?(it[:from][:x]) }
