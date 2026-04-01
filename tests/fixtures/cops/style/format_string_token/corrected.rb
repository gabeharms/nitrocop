x = '%<name>s is %<age>s'
y = format('%s %s %d', a, b, c)
z = '%<greeting>s %<target>s'
w = sprintf('%s %s', a, b)
v = <<~HEREDOC
  hello %<name>s
  world %<age>s
HEREDOC
# Template tokens in regular strings used with redirect
a1 = "admin/customize/watched_words/%<path>s"
a2 = "tag/%<tag_id>s"
# Unannotated tokens in format context with % operator
a3 = "items/%s/%s...%s" % [file, ver1, ver2]
