'foo'.gsub("bar", 'baz')

'foo'.sub("bar", 'baz')

'foo'.split(",")

'foo'.gsub(".", '-')

'foo'.split("-")

'foo'.sub("/", '-')

# Empty regexp is deterministic
'foo'.split("")

# %r with slash delimiter is deterministic
'foo'.gsub(".", '-')

# %r with ! delimiter is deterministic
'foo'.split("foo")
