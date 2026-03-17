items.each { |x| puts x }

items.each { |x| puts x }

items.each { |x| puts x }

items.each { |x| puts x }

handler = proc {|s| cmd.call s}

result = ->(x, y) { puts x }

result = ->(x, y) { puts x }

result = ->(a, b, c) { puts a }

items.each { |x, y| puts x }

items.each { |a, (x, y), z| puts x }

items.each { |a, (x, y), z| puts x }

[1].each {|foo| foo }

[1].each {|glark| 1}

1.times do |a|
