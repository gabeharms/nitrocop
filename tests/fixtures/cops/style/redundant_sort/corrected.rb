[2, 1, 3].min

[2, 1, 3].max

[2, 1, 3].min

[2, 1, 3].max

[2, 1, 3].min

[2, 1, 3].max

foo.min_by { |x| x.length }

foo.max_by { |x| x.length }

foo.min_by(&:name)

foo.max_by(&:name)

foo.max { |a, b| b <=> a }

foo.min { |a, b| a <=> b }

items.min { |a, b| a.name <=> b.name }

items
  .min_by { |x| x.name }

items
  .max_by { |x| x.name }

items
  .max { |a, b| a.score <=> b.score }
