[].each do |o|
  next unless o == 1
  puts o
  puts o
  puts o
end

3.downto(1) do
  next unless true
  a = 1
  b = 2
  c = 3
end

items.map do |item|
  next if item.nil?
  process(item)
  transform(item)
  finalize(item)
end
