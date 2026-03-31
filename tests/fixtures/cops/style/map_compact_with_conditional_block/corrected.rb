# If with no else (implicit nil) — modifier form
ary.filter_map { |x| x if x > 1 }
list.filter_map { |item| item if item.valid? }
[1, 2, 3].filter_map { |n| n if n.odd? }

# If with no else — block form
ary.filter_map do |x|
  if x > 1
    x
  end
end

# Unless modifier form (reject pattern)
ary.filter_map { |item| item unless item.bar? }

# Unless block form (reject pattern)
ary.filter_map do |item|
  unless item.bar?
    item
  end
end

# If with else=next
ary.filter_map do |item|
  if item.bar?
    item
  else
    next
  end
end

# If with then=next (reject pattern)
ary.filter_map do |item|
  if item.bar?
    next
  else
    item
  end
end

# Ternary: select pattern
foo.filter_map { |item| item.bar? ? item : next }

# Ternary: reject pattern
foo.filter_map { |item| item.bar? ? next : item }

# Guard clause: next if (reject)
ary.filter_map do |item|
  next if item.bar?

  item
end

# Guard clause: next unless (select)
ary.filter_map do |item|
  next unless item.bar?

  item
end

# Guard clause: next item if (select with value)
ary.filter_map do |item|
  next item if item.bar?
end

# Guard clause: next item unless (reject with value)
ary.filter_map do |item|
  next item unless item.bar?
end

# next item if + nil (select with value and nil return)
ary.filter_map do |item|
  next item if item.bar?

  nil
end

# next item unless + nil (reject with value and nil return)
ary.filter_map do |item|
  next item unless item.bar?

  nil
end

# If with next item in then branch and nil in else (select)
ary.filter_map do |item|
  if item.bar?
    next item
  else
    nil
  end
end

# If with nil in then branch and next item in else (reject)
ary.filter_map do |item|
  if item.bar?
    nil
  else
    next item
  end
end

# filter_map with if/next
ary.filter_map do |item|
  if item.bar?
    item
  else
    next
  end
end

# filter_map with modifier if
ary.filter_map { |item| item if item.bar? }

# Guard clause: next nil if + item (select with nil guard)
ary.filter_map do |item|
  next nil if item.bar?

  item
end

# Guard clause: next nil unless + item (reject with nil guard)
ary.filter_map do |item|
  next nil unless item.bar?

  item
end

# filter_map in multi-line method chain (receiver on different line)
foo.bar
  .filter_map { |x| x if x.valid? }
