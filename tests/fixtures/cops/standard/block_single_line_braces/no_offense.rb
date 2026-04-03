items.each { |item| item / 5 }

[1, 2, 3].select { |n| n.odd? }

items.map do |x|
  x.to_s
end

items.each do |item|
  process(item)
end
