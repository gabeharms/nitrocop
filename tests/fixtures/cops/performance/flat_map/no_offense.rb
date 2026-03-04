[1, 2, 3].flat_map { |x| [x, x] }
[1, 2, 3].flatten
[1, 2, 3].map { |x| x }.compact
arr.map { |x| x }.first
arr.collect.flatten
Parallel.map(items, opts, &worker).flatten(1)
TaskRunner.collect(batches, config, &block).flatten
array.map { |x| [x, x] }.flatten(2)
array.collect { |x| [x, x] }.flatten(3)
array.map { |x| [x, x] }.flatten!(2)
array.collect { |x| [x, x] }.flatten!(3)
[1, 2, 3].map { |x| [x, x] }.flatten
[1, 2, 3].collect { |x| [x, x] }.flatten
arr.map { |item| item.children }.flatten
items.collect(&:children).flatten
[1, 2, 3].map { |x| [x, x] }.flatten!
items.collect { |x| [x, x] }.flatten!
