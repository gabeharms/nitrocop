class Foo
  end
  ^^^ Layout/EndAlignment: Align `end` with `class`.

module Bar
  end
  ^^^ Layout/EndAlignment: Align `end` with `module`.

if true
  1
  end
  ^^^ Layout/EndAlignment: Align `end` with `if`.

while true
  break
  end
  ^^^ Layout/EndAlignment: Align `end` with `while`.

case x
when 1
  :a
  end
  ^^^ Layout/EndAlignment: Align `end` with `case`.

unless false
  1
  end
  ^^^ Layout/EndAlignment: Align `end` with `unless`.

until done
  work
  end
  ^^^ Layout/EndAlignment: Align `end` with `until`.

class << self
  def foo; end
  end
  ^^^ Layout/EndAlignment: Align `end` with `class`.

case [1, 2]
in [a, b]
  a + b
  end
  ^^^ Layout/EndAlignment: Align `end` with `case`.
