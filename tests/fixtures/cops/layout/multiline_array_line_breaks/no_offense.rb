x = [
  :a,
  :b,
  :c
]

y = [:a, :b, :c]

z = [
  1,
  2,
  3
]

# All elements on same line, separate from brackets
w = [
  1, 2, 3,
]

# Single element multiline array
s = [
  :only_one
]

# Rescue with single exception
begin
  something
rescue FooError
  retry
end

# Rescue with each exception on its own line
begin
  something
rescue FooError,
       BarError,
       BazError
  retry
end

# Rescue with all exceptions on same line
begin
  something
rescue FooError, BarError, BazError
  retry
end

# Implicit array — all on same line
a, b, c = val1, val2, val3

# Implicit array — each on own line
a, b, c =
  val1,
  val2,
  val3

# Method call with implicit array all on same line
config.cache_store = :redis_cache_store, { url: "redis://localhost" }

# Constant assignment implicit array on single line
ITEMS = :scan, :skip, :match
