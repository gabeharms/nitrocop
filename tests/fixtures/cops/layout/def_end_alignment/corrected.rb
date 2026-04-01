def foo
  42
end
def bar(x)
  x + 1
end
  def baz
    42
  end
# Non-modifier mid-line def: end must align with def keyword
false && def guarded
         end
# Parenthesized def: end must align with def keyword
protected (def parens_method
  1
           end)
# Semicolon-separated mid-line def: end must align with def keyword
module Helpers; def compute(x)
  x + 1
                end
