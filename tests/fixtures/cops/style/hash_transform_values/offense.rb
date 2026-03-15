# each_with_object patterns
x.each_with_object({}) { |(k, v), h| h[k] = foo(v) }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `each_with_object`.

x.each_with_object({}) { |(k, v), h| h[k] = v.to_s }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `each_with_object`.

x.each_with_object({}) { |(k, v), h| h[k] = v.to_i }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `each_with_object`.

x.each_with_object({}) do |(k, v), h|
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `each_with_object`.
  h[k] = v * 2
end

# Hash[_.map {...}] pattern
Hash[x.map { |k, v| [k, foo(v)] }]
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `Hash[_.map {...}]`.

Hash[x.collect { |k, v| [k, v.to_s] }]
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `Hash[_.map {...}]`.

# _.map {...}.to_h pattern
x.map { |k, v| [k, v.to_s] }.to_h
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `map {...}.to_h`.

x.collect { |k, v| [k, v.to_i] }.to_h
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `map {...}.to_h`.

# _.to_h {...} pattern
x.to_h { |k, v| [k, v.to_s] }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `to_h {...}`.

x.to_h { |k, v| [k, foo(v)] }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/HashTransformValues: Prefer `transform_values` over `to_h {...}`.
