dest = src.map { |x| x * 2 }
items.map { |item| item.to_s }
src.each { |x| process(x) }
src.each { |x| puts x }
src.each { |x| x.save; log(x) }
src.each { |e| @dest << e }
self.each { |e| dest << e }
each { |e| dest << e }
CSV.generate do |csv|
  items.each { |item| csv << [item.name] }
end
items.each { |item| output << item.to_s }
dest = "hello"
src.each { |e| dest << e.to_s }
# Variable used between init and each - not a pure map pattern
attributes = []
attributes << "Name: #{record.name}"
attributes << "Email: #{record.email}"
records.each do |record|
  attributes << "#{record.key}: #{record.value}"
end
# Variable used inside an intermediate assignment's value expression
order = []
entries = src.map do |entry|
  order << entry.full_name
  transform(entry)
end
entries.each do |entry|
  order << entry.path
end
# Safe navigation on receiver - not flagged
results = []
collection&.each do |item|
  results << item.to_s
end
# Operator assignment (+=) between init and each
attachments = []
attachments += existing_items(list)
items.each do |item|
  attachments << { data: item.data, name: item.name }
end
# Or-assignment (||=) between init and each
values = []
values ||= defaults
src.each { |e| values << e }
