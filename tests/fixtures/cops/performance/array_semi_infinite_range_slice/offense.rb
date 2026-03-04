arr[2..]
^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `[]` with a semi-infinite range.
array[2..]
^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `[]` with a semi-infinite range.
array[2...]
^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `[]` with a semi-infinite range.
array[..2]
^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `take` instead of `[]` with a semi-infinite range.
array[...2]
^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `take` instead of `[]` with a semi-infinite range.
array.slice(2..)
^^^^^^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `slice` with a semi-infinite range.
array.slice(..2)
^^^^^^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `take` instead of `slice` with a semi-infinite range.
arr[0x1f0..]
^^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `[]` with a semi-infinite range.
arr[0b1010..]
^^^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `[]` with a semi-infinite range.
arr[0o77..]
^^^^^^^^^^^ Performance/ArraySemiInfiniteRangeSlice: Use `drop` instead of `[]` with a semi-infinite range.
