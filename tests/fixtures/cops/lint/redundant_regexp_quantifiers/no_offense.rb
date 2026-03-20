foo = /(?:ab+)+/
foo = /(?:a|b+)+/
foo = /(?:a+|b)+/
foo = /(?:\d\D+)+/
foo = /(a+)+/
foo = /(?:(a+))+/
foo = /(?:(a)+)+/
foo = /(?:a++)+/
foo = /(?:a+)++/
foo = /(?:a+?)+/
foo = /(?:a+)+?/
foo = /(?:a{3,4})+/
foo = /(?:a+){3,4}/
foo = /a{3,4}?/
foo = /a{2,}?/
foo = /a{0,3}?/
# Unicode property escapes with quantifiers are not redundant
foo = /(\p{Pd}?\d){10}$/
foo = /^\p{Pd}?\d+\p{Pd}?$/
foo = /\p{L}+/
foo = /\p{Nd}?/
# Unicode codepoint escapes
foo = /\u{FEFF}?\s*/
# Named groups with lazy quantifiers
foo = /(?<name>\w+?)\s+/
