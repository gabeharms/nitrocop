x =~ /=/

x =~ /:/

x =~ /,/

# Inside character class: dot is redundant
x =~ /[.]/
# Inside character class: plus is redundant
x =~ /[+]/
# Escaped hyphen at end of character class is redundant
x =~ /[a-z0-9-]/
