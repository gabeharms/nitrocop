dest = []
dest = src.map { |x| x * 2 }
result = []
result = items.map { |item| item.to_s }
output = []
output = list.map { |e| transform(e) }
values = Array.new
values = src.map { |e| e.to_s }
data = Array[]
data = src.map { |e| e * 2 }
# [].tap pattern with each and <<
[].tap { |res| src.each { |v| res << v } }
# [].tap with do...end block and each inside
[].tap do |files|
  directory.files.each { |file| files << file }
end
# [].tap with push instead of <<
[].tap { |values| keys.each { |key| values << items[key] } }
