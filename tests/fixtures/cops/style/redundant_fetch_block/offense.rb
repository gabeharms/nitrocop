hash.fetch(:key) { 5 }
     ^^^^^^^^^^^^^^^^^ Style/RedundantFetchBlock: Use `fetch(:key, 5)` instead of `fetch(:key) { 5 }`.

hash.fetch(:key) { :value }
     ^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantFetchBlock: Use `fetch(:key, :value)` instead of `fetch(:key) { :value }`.

hash.fetch(:key) { 'default' }
     ^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantFetchBlock: Use `fetch(:key, 'default')` instead of `fetch(:key) { 'default' }`.

# String default without frozen_string_literal should still be flagged
val = hash.fetch(:key) { 'fallback' }
           ^^^^^^^^^^^^^^^^^^^^^^^^^ Style/RedundantFetchBlock: Use `fetch(:key, 'fallback')` instead of `fetch(:key) { 'fallback' }`.
