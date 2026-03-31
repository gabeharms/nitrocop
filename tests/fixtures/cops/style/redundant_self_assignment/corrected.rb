arr.sort!

hash.merge!(other)

arr.concat(other)

arr.push(item)

arr.append(item)

str.replace('new')

arr.reverse!

arr.collect! { |x| x + 1 }

arr.map! { |x| x + 1 }

arr.delete_if { |x| x > 1 }

arr.keep_if { |x| x > 1 }

hash.update(other)

hash.transform_keys! { |k| k.to_s }

hash.transform_values! { |v| v + 1 }

arr.prepend(item)

arr.clear

arr.rotate!(2)

arr.shuffle!

arr.sort_by! { |x| x }

arr.fill(0)

arr.insert(0, item)

arr.unshift(item)

hash.rehash

hash.compare_by_identity

@foo.concat(ary)

@@foo.concat(ary)

$foo.concat(ary)

other.foo.concat(ary)
