# Already using transform_values
x.transform_values { |v| foo(v) }

# Noop transformation: value unchanged — likely not a hash
x.each_with_object({}) { |(k, v), h| h[k] = v }

# Key is transformed, not values — this is transform_keys territory
x.each_with_object({}) { |(k, v), h| h[k.to_sym] = foo(v) }

# Block shorthand
x.transform_values(&:to_s)

# Value expression references the key variable — not a transform_values candidate
group_columns.each_with_object({}) do |(aliaz, col_name), types|
  types[aliaz] = col_name.try(:type_caster) || fetch(aliaz)
end

# Non-destructured block params — iterating an array/enumerable, not a hash
items.each_with_object({}) { |item, result| result[item] = true }

# Non-destructured do..end block params
records.each_with_object({}) do |record, memo|
  memo[record] = [record.name, record.id]
end

# Value assigned is the memo variable itself — not a transform
x.each_with_object({}) { |(k, v), h| h[k] = h }

# Array receiver — can't be simplified
[1, 2, 3].each_with_object({}) { |(k, v), h| h[k] = v.to_s }

# map.to_h where key is also transformed — not a transform_values candidate
x.map { |k, v| [k.to_sym, v.to_s] }.to_h

# Hash[] where both key and value are transformed
Hash[x.map { |k, v| [k.to_sym, v.to_s] }]

# to_h where both key and value are transformed
x.to_h { |k, v| [k.to_sym, v.to_s] }

# Noop — value is just the value
x.map { |k, v| [k, v] }.to_h

# map/to_h with value transformation that uses the key
x.map { |k, v| [k, "#{k}: #{v}"] }.to_h

# to_h with value that uses key
x.to_h { |k, v| [k, v + k] }

# each_with_object with non-empty hash argument
x.each_with_object(defaults) { |(k, v), h| h[k] = v.to_s }

# map.to_h where body isn't an array pair
x.map { |k, v| k }.to_h

# each_with_object body references memo in the value expression
x.each_with_object({}) { |(k, v), h| h[k] = h.size + v }
