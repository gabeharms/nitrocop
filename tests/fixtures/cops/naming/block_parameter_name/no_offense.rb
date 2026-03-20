arr.each { |x| x }
arr.map { |item| item }
arr.each { |e| e }
arr.select { |i| i.valid? }
arr.each_with_object({}) { |(k, v), h| h[k] = v }
arr.each { |_| nil }
arr.map { |_, v| v }
foo { |name:, age: nil| [name, age] }
