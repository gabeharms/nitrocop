x = 1


^ Layout/EmptyLines: Extra blank line detected.
y = 2


^ Layout/EmptyLines: Extra blank line detected.

^ Layout/EmptyLines: Extra blank line detected.
z = 3

# Consecutive blank lines between code and trailing comments
a = 1


^ Layout/EmptyLines: Extra blank line detected.
# trailing comment

b = 2


^ Layout/EmptyLines: Extra blank line detected.
# another comment

# Consecutive blank lines inside =begin/=end blocks ARE flagged.
=begin
some docs


^ Layout/EmptyLines: Extra blank line detected.
more docs
=end
p 1

# Consecutive blank lines in a comment-only file
# frozen_string_literal: true


^ Layout/EmptyLines: Extra blank line detected.
# Another comment
