# ... forwarding: both *rest and **kwrest present
def foo(...)
  bar(...)
end

def test(...)
  other(...)
end

def forward_triple_to_super(...)
  super(...)
end

# ... forwarding: both *rest and **kwrest with leading positional param
def method_missing(m, ...)
  @tpl.send(m, ...)
end

# Anonymous block forwarding (&block -> &) — block only
def run_task(&)
  executor.post(&)
end

# Anonymous block forwarding with extra positional args
def handle(name, &)
  registry.call(name, &)
end

# Anonymous rest + block forwarding with extra positional args
def dispatch(x, *, &)
  handler.run(x, *, &)
end

# ... forwarding with leading args in call site (both *rest and **kwrest)
def before_action(...)
  set_callback(:action, :before, ...)
end

# Anonymous forwarding with yield
def foo_yield(*)
  yield(*)
end

# Anonymous kwargs forwarding with yield
def bar_yield(**)
  yield(**)
end

