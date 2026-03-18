a = *[1, 2, 3]
    ^^^^^^^^^^ Lint/RedundantSplatExpansion: Replace splat expansion with comma separated values.

a = *'a'
    ^^^^ Lint/RedundantSplatExpansion: Replace splat expansion with comma separated values.

a = *1
    ^^ Lint/RedundantSplatExpansion: Replace splat expansion with comma separated values.

# Percent literal splat inside an array literal (method arg) — NOT exempt
foo([*%w[a b c]])
     ^^^^^^^^^ Lint/RedundantSplatExpansion: Pass array contents as separate arguments.

bar([*%W[x y], z])
     ^^^^^^^ Lint/RedundantSplatExpansion: Pass array contents as separate arguments.

baz([*%i[a b c]])
     ^^^^^^^^^ Lint/RedundantSplatExpansion: Pass array contents as separate arguments.
