arr = arr.sort!
^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `sort!` modifies `arr` in place.

hash = hash.merge!(other)
^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `merge!` modifies `hash` in place.

arr = arr.concat(other)
^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `concat` modifies `arr` in place.

arr = arr.push(item)
^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `push` modifies `arr` in place.

arr = arr.append(item)
^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `append` modifies `arr` in place.

str = str.replace('new')
^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `replace` modifies `str` in place.

arr = arr.reverse!
^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `reverse!` modifies `arr` in place.

arr = arr.collect! { |x| x + 1 }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `collect!` modifies `arr` in place.

arr = arr.map! { |x| x + 1 }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `map!` modifies `arr` in place.

arr = arr.delete_if { |x| x > 1 }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `delete_if` modifies `arr` in place.

arr = arr.keep_if { |x| x > 1 }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `keep_if` modifies `arr` in place.

hash = hash.update(other)
^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `update` modifies `hash` in place.

hash = hash.transform_keys! { |k| k.to_s }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `transform_keys!` modifies `hash` in place.

hash = hash.transform_values! { |v| v + 1 }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `transform_values!` modifies `hash` in place.

arr = arr.prepend(item)
^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `prepend` modifies `arr` in place.

arr = arr.clear
^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `clear` modifies `arr` in place.

arr = arr.rotate!(2)
^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `rotate!` modifies `arr` in place.

arr = arr.shuffle!
^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `shuffle!` modifies `arr` in place.

arr = arr.sort_by! { |x| x }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `sort_by!` modifies `arr` in place.

arr = arr.fill(0)
^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `fill` modifies `arr` in place.

arr = arr.insert(0, item)
^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `insert` modifies `arr` in place.

arr = arr.unshift(item)
^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `unshift` modifies `arr` in place.

hash = hash.rehash
^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `rehash` modifies `hash` in place.

hash = hash.compare_by_identity
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `compare_by_identity` modifies `hash` in place.

@foo = @foo.concat(ary)
^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `concat` modifies `@foo` in place.

@@foo = @@foo.concat(ary)
^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `concat` modifies `@@foo` in place.

$foo = $foo.concat(ary)
^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `concat` modifies `$foo` in place.

other.foo = other.foo.concat(ary)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantSelfAssignment: Redundant self-assignment. `concat` modifies `foo` in place.
