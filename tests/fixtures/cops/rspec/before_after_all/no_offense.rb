before(:each) { do_something }
before(:example) { do_something }
after(:each) { do_something }
after(:example) { do_something }
before { do_something }
after { do_something }
config.before(:each) { do_something }
config.after(:example) { do_something }

# block_pass (&proc) adds an extra arg in Parser AST, so RuboCop doesn't match
@state.before(:all, &@proc)
proxy.before(:all, &callback)
proxy.after(:all, &callback)
parent.after(:all, &@c)

# Extra arguments beyond :all — RuboCop requires exactly 1 arg
config.before(:all, :with_monkey_patches => scope) { setup }
config.before(:all, type: :integration) do
  setup
end
config.before(:all, type: :controller) do
  setup
end
