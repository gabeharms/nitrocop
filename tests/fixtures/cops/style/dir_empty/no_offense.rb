Dir.empty?("path/to/dir")
Dir.exist?("path/to/dir")
Dir.entries("path/to/dir")
Dir.children("path/to/dir")
Dir.each_child("path/to/dir") { |c| puts c }
# Non-matching integer values should not flag
Dir.entries('path/to/dir').size == 5
Dir.children('path/to/dir').size == 3
Dir.entries('path/to/dir').size != 5
Dir.children('path/to/dir').size > 3

# length and count are not matched by RuboCop, only size
Dir.entries('path/to/dir').length == 2
Dir.entries('path/to/dir').count == 2
Dir.children('path/to/dir').length == 0
Dir.children('path/to/dir').count == 0
x = 1
