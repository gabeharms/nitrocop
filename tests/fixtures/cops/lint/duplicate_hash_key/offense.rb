hash = { a: 1, b: 2, a: 3 }
                     ^^ Lint/DuplicateHashKey: Duplicated key in hash literal.

hash = { 'x' => 1, 'y' => 2, 'x' => 3 }
                             ^^^ Lint/DuplicateHashKey: Duplicated key in hash literal.

hash = { 1 => :a, 2 => :b, 1 => :c }
                           ^ Lint/DuplicateHashKey: Duplicated key in hash literal.

# Multiplication is a literal-preserving operator (in RuboCop's LITERAL_RECURSIVE_METHODS)
hash = { (2 * 3) => :a, (2 * 3) => :b }
                        ^^^^^^^ Lint/DuplicateHashKey: Duplicated key in hash literal.
