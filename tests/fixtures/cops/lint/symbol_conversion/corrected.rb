# Unnecessary to_sym on symbol literal
:foo

# Unnecessary to_sym on string literal
:foo

# Unnecessary to_sym on string with underscores
:foo_bar

# Unnecessary to_sym on string requiring quoting
:"foo-bar"

# Unnecessary intern on symbol literal
:foo

# Unnecessary intern on string literal
:foo

# Unnecessary intern on string with underscores
:foo_bar

# Unnecessary intern on string requiring quoting
:"foo-bar"

# Unnecessarily quoted standalone symbol (double quotes)
:foo

# Unnecessarily quoted standalone symbol (double quotes, underscore)
:foo_bar

# Unnecessarily quoted standalone symbol (single quotes)
:foo

# Unnecessarily quoted standalone symbol (single quotes, underscore)
:foo_bar

# Unnecessarily quoted operator symbol
obj.send(:+)

# Unnecessarily quoted instance variable symbol
instance_variable_get :@ivar

# Quoted hash key (string style)
{ name: 'val' }

# Quoted hash key (double-quoted string style)
{ role: 'val' }

# Multiple quoted hash keys
{ status: 1, color: 2 }

# Quoted symbol as hash value
{ foo: :bar }

# Quoted symbol as hash key (rocket style)
{ :foo => :bar }

# Quoted hash key ending with !
{ foo!: 'bar' }

# Quoted hash key ending with ?
{ foo?: 'bar' }

# Interpolated string to_sym
:"foo-#{bar}"

# Interpolated string intern
:"foo-#{bar}"

# Uppercase quoted hash key
{ Foo: 1 }

# Double-quoted uppercase hash key
{ Bar: 1 }

# Quoted hash key with underscore prefix
{ _private: 1 }

# Unnecessarily quoted numeric global variable symbol
:$1

# Unnecessarily quoted special global variable symbol
:$?

# Unnecessarily quoted special global symbol ($!)
:$!

# UTF-8 symbol that can be unquoted (Ruby allows multi-byte identifiers)
:résumé

# UTF-8 single-quoted symbol
:café

# UTF-8 hash key (colon-style)
{ naïve: 1 }

# Percent-string notation with interpolation and .to_sym
:"cover_#{face}_image"

# Percent-string notation with leading interpolation and .to_sym
:"#{periphery}_background_color"

# Percent-string notation with interpolation and .intern
:"prefix_#{name}"

# Non-ASCII standalone symbol that can be unquoted (multiplication sign)
:×

# Special global variable $$ (process ID)
:$$

# Non-UTF8 string to_sym
:"\xFF"

# Percent-s with double-quote delimiter
:test
