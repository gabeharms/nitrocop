foo.none?
foo.any?
foo.odd?
x != false
items.reject { |x| x.valid? }
items.select { |k, v| v == :active }
items.reject! { |x| x.empty? }
items.select! { |k, v| v == :a }
items.select do |x|
  x.nil?
end
x == y
x =~ /pattern/
foo.none?
foo&.reject { |e| e }
