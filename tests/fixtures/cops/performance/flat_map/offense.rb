ancestors.map(&:instance_methods).flatten(1)
          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Performance/FlatMap: Use `flat_map` instead of `map...flatten`.
items.collect(&:children).flatten(1)
      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Performance/FlatMap: Use `flat_map` instead of `collect...flatten`.
[1, 2, 3].map { |x| [x, x] }.flatten(1)
          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Performance/FlatMap: Use `flat_map` instead of `map...flatten`.
items.collect { |x| [x, x] }.flatten!(1)
      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Performance/FlatMap: Use `flat_map` instead of `collect...flatten!`.
ancestors.map(&:instance_methods).flatten!(1)
          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Performance/FlatMap: Use `flat_map` instead of `map...flatten!`.
ancestors.reject { |klass| klass == self }
  .map(&:instance_methods).flatten(1)
   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Performance/FlatMap: Use `flat_map` instead of `map...flatten`.
