%r_ls_
^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.

%r{foo}
^^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.

%r(bar)
^^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.

/foo\/bar/
^^^^^^^^^^ Style/RegexpLiteral: Use `%r` around regular expression.

# %r with space/eq content NOT in a method argument context should still offend
x = %r( foo )
    ^^^^^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.

CONST = %r{ pattern }mix
        ^^^^^^^^^^^^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.

sep = %r/ +\| +/
      ^^^^^^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.

val = %r{=pattern}
      ^^^^^^^^^^^^ Style/RegexpLiteral: Use `//` around regular expression.
