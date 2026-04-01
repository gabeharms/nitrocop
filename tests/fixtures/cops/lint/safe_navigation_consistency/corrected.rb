foo.bar && foo.baz
foo&.bar && foo.baz
foo.bar || foo.baz
foo&.bar || foo&.baz
foo.bar && foobar.baz && foo.qux
foo.bar || foobar.baz || foo.qux
foo&.bar && foo.baz || foo&.qux
foo.bar && foo.baz || foo.qux
foo&.bar && foo.baz || foo.qux
foo > 5 && foo.zero?
foo.bar && foo.baz = 1
foo&.bar && foo.baz = 1
