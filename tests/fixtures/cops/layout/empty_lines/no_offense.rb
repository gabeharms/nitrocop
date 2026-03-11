x = 1

y = 2

z = 3
a = 4

b = 5

# Whitespace-only lines are NOT blank according to RuboCop.
# The following lines contain spaces/tabs but are not truly empty.
def foo
  x = 1
  
  
  y = 2
end

# Consecutive blank lines inside a multi-line string are not offenses.
result = "test


                                    string"

# Consecutive blank lines inside =begin/=end block comments are not offenses.
=begin
some documentation


more documentation
=end
x = 1

# Blank lines after the last code line are not checked by RuboCop,
# which uses token-based gap detection and stops after the last token.
puts "last code"


# This trailing comment section has consecutive blank lines above,
# but RuboCop does not flag them because they're after the last token.
