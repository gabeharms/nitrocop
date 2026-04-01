# Exploded style (default): flag raise/fail with Error.new(single_arg)
raise RuntimeError, "message"

raise ArgumentError, "bad argument"

fail StandardError, "oops"

raise RuntimeError

raise Foo::Bar, "message"
