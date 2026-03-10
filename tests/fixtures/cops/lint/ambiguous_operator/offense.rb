do_something *some_array
             ^ Lint/AmbiguousOperator: Ambiguous splat operator. Parenthesize the method arguments if it's surely a splat operator, or add a whitespace to the right of the `*` if it should be a multiplication.
foo *bar
    ^ Lint/AmbiguousOperator: Ambiguous splat operator. Parenthesize the method arguments if it's surely a splat operator, or add a whitespace to the right of the `*` if it should be a multiplication.
method *args
       ^ Lint/AmbiguousOperator: Ambiguous splat operator. Parenthesize the method arguments if it's surely a splat operator, or add a whitespace to the right of the `*` if it should be a multiplication.
do_something +42
             ^ Lint/AmbiguousOperator: Ambiguous positive number operator. Parenthesize the method arguments if it's surely a positive number operator, or add a whitespace to the right of the `+` if it should be an addition.
do_something -42
             ^ Lint/AmbiguousOperator: Ambiguous negative number operator. Parenthesize the method arguments if it's surely a negative number operator, or add a whitespace to the right of the `-` if it should be a subtraction.
do_something &block
             ^ Lint/AmbiguousOperator: Ambiguous block operator. Parenthesize the method arguments if it's surely a block operator, or add a whitespace to the right of the `&` if it should be a binary AND.
do_something **kwargs
             ^^ Lint/AmbiguousOperator: Ambiguous keyword splat operator. Parenthesize the method arguments if it's surely a keyword splat operator, or add a whitespace to the right of the `**` if it should be an exponent.
