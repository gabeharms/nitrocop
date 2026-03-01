str.delete_suffix('foo')
str.gsub(/foo/, '')
str.gsub(/foo\z/, 'bar')
str.gsub(/.*foo\z/, '')
str.sub(/fo+\z/, '')
# Regex with flags — can't use delete_suffix
str.gsub(/suffix\z/i, '')
str.sub(/suffix\z/m, '')
str.gsub(/suffix\z/x, '')
# Escaped regex metacharacters that are NOT safe literals
str.gsub(/@suffix\z/, '')
# Both \A and \z anchors — not just a suffix
str.gsub(/\Asuffix\z/, '')
# Unescaped dot is a regex metacharacter, not a literal suffix
str.sub(/.rb\z/, '')
str.gsub(/.git\z/, '')
str.sub!(/.\z/, '')
