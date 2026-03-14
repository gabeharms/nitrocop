def foo
  42
  end
  ^^^ Layout/DefEndAlignment: Align `end` with `def`.
def bar(x)
  x + 1
    end
    ^^^ Layout/DefEndAlignment: Align `end` with `def`.
  def baz
    42
      end
      ^^^ Layout/DefEndAlignment: Align `end` with `def`.
# Non-modifier mid-line def: end must align with def keyword
false && def guarded
end
^^^ Layout/DefEndAlignment: Align `end` with `def`.
# Parenthesized def: end must align with def keyword
protected (def parens_method
  1
end)
^^^ Layout/DefEndAlignment: Align `end` with `def`.
# Semicolon-separated mid-line def: end must align with def keyword
module Helpers; def compute(x)
  x + 1
end
^^^ Layout/DefEndAlignment: Align `end` with `def`.
