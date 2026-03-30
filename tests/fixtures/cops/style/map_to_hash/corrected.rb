foo.to_h { |x| [x, x * 2] }

foo.to_h { |x, y| [x.to_s, y.to_i] }

items.to_h { |k, v| [k, v * 2] }

foo&.to_h(&:do_something)

foo&.to_h(&:do_something)
