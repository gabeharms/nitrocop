"hello #{}"
       ^^^ Lint/EmptyInterpolation: Empty interpolation detected.

"foo #{} bar"
     ^^^ Lint/EmptyInterpolation: Empty interpolation detected.

"#{}"
 ^^^ Lint/EmptyInterpolation: Empty interpolation detected.

"value #{''}"
       ^^^^^ Lint/EmptyInterpolation: Empty interpolation detected.

"nil #{nil}"
     ^^^^^^ Lint/EmptyInterpolation: Empty interpolation detected.

/regexp #{}/
        ^^^ Lint/EmptyInterpolation: Empty interpolation detected.

`backticks #{nil}`
           ^^^^^^ Lint/EmptyInterpolation: Empty interpolation detected.
