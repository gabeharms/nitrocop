x = ?x
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

y = ?a
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

z = ?Z
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

w = ?\n
    ^^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

# Multi-byte Unicode character literals (2 chars in source: ? + char)
a = ?é
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

b = ?中
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

c = ?λ
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.

d = ?𝄞
    ^^ Style/CharacterLiteral: Do not use the character literal - use string literal instead.
