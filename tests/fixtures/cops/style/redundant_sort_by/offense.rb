array.sort_by { |x| x }
      ^^^^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { |x| x }`.

array.sort_by { |y| y }
      ^^^^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { |y| y }`.

array.sort_by do |x|
      ^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { |x| x }`.
  x
end

array.sort_by { _1 }
      ^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { _1 }`.

array&.sort_by { _1 }
       ^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { _1 }`.

array.sort_by { it }
      ^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { it }`.

array&.sort_by { it }
       ^^^^^^^^^^^^^^ Style/RedundantSortBy: Use `sort` instead of `sort_by { it }`.
