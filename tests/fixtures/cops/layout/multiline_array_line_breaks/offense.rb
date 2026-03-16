x = [
  :a, :b,
      ^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  :c
]

y = [
  1, 2,
     ^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  3
]

z = [
  :foo, :bar,
        ^^^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  :baz
]

# Rescue exception lists — multiple exceptions on same line in multi-line rescue
begin
  something
rescue FooError, BarError,
                 ^^^^^^^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
       BazError
  retry
end

begin
  something
rescue FooError, BarError,
                 ^^^^^^^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
       BazError, QuxError
                 ^^^^^^^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  retry
end

# Implicit array in multi-assignment (no brackets)
a, b, c =
  val1, val2,
        ^^^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  val3

# Method call with implicit array args (e.g. config.cache_store=)
config.cache_store = :redis_cache_store, {
                                         ^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  url: "redis://localhost:6379/1",
  expires_in: 90
}

# Constant assignment with implicit array
ITEMS = :scan, :skip,
               ^^^^^ Layout/MultilineArrayLineBreaks: Each item in a multi-line array must start on a separate line.
  :match
