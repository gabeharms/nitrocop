it_behaves_like :foo_bar_baz
                ^^^^^^^^^^^^ RSpec/SharedExamples: Prefer 'foo bar baz' over `:foo_bar_baz` to titleize shared examples.
shared_examples :some_thing do
                ^^^^^^^^^^^ RSpec/SharedExamples: Prefer 'some thing' over `:some_thing` to titleize shared examples.
end
include_examples :hello_world
                 ^^^^^^^^^^^^ RSpec/SharedExamples: Prefer 'hello world' over `:hello_world` to titleize shared examples.
shared_context :my_context do
               ^^^^^^^^^^^ RSpec/SharedExamples: Prefer 'my context' over `:my_context` to titleize shared examples.
end
include_context :test_setup
                ^^^^^^^^^^^ RSpec/SharedExamples: Prefer 'test setup' over `:test_setup` to titleize shared examples.
RSpec.shared_context :rspec_context do
                     ^^^^^^^^^^^^^^ RSpec/SharedExamples: Prefer 'rspec context' over `:rspec_context` to titleize shared examples.
end
