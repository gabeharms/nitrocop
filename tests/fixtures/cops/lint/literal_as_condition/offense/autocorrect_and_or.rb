if 1 && x
   ^ Lint/LiteralAsCondition: Literal `1` appeared as a condition.
  top
end

if nil || x
   ^^^ Lint/LiteralAsCondition: Literal `nil` appeared as a condition.
  top
end

def foo
  bar if 1 && return
         ^ Lint/LiteralAsCondition: Literal `1` appeared as a condition.
end
