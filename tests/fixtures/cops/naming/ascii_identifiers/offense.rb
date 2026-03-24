def résumé
     ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.
  nil
end

def café
       ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.
  nil
end

naïve = 1
  ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.

älg = 1
^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.

foo∂∂bar = baz
   ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.

alias now_in_microseconds now_in_μs
                                 ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.

# Method definitions with non-ASCII and ? or ! suffix: in Ruby's lexer,
# `def` context produces tIDENTIFIER (not tFID), so RuboCop flags these.
def non_è_un?(arg)
        ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.
  !is_a? arg
end

def è_un_commento?
    ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.
  false
end

def è_una_stringa!
    ^ Naming/AsciiIdentifiers: Use only ascii symbols in identifiers.
  true
end
