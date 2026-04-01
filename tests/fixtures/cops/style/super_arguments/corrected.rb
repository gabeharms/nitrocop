def foo(a, b)
  super
end

def bar(x, y)
  super
end

def baz(name)
  super
end

def with_rest(*args, **kwargs)
  super
end

def with_keyword(name:, age:)
  super
end

def with_block(name, &block)
  super
end

def with_mixed(a, *args, b:)
  super
end

# super() with no-arg def
def no_args
  super
end

# super(...) forwarding
def with_forwarding(...)
  super
end

# Block-only forwarding
def block_only(&blk)
  super
end

# super with block literal should still flag when args match
def with_block_literal(a, b)
  super { x }
end

# super with block literal and non-forwarded block arg should flag
def with_block_arg_and_literal(a, &blk)
  super { x }
end

# Ruby 3.1 hash value omission: super(type:, hash:)
def with_shorthand_keywords(type:, hash:)
  super
end

# Nested def — inner super should flag for inner def
def outer(a)
  def inner(b:)
    super
  end
  super
end

# Anonymous block forwarding with named keyword rest
def with_named_kwargs_and_anonymous_block(filter: /UserAudit/, level: :info, **args, &)
  super
end

# Anonymous block forwarding with positional arguments
def with_args_and_anonymous_block(level, index, message, payload, exception, &)
  super
end

# Anonymous positional and keyword forwarding
def with_anonymous_rest_and_keywords(*, **)
  super if defined?(super)
end

# Anonymous block forwarding after local mutation
def with_mutation_before_anonymous_block(xml, &)
  xml = strip_whitespace(xml)
  super
end

# Post arguments after rest params should keep Ruby's order
def with_post_arg(ep, *params, configs)
  super
end

# Anonymous block param with an inline block still reports the positional/keyword message
def with_inline_block_and_anonymous_block(name, *args, &)
  super do
    instance_eval(&)
  end
end

# Anonymous keyword rest forwarding
def with_anonymous_keyword_rest(quantity: nil, hard_limit: INFINITY, soft_limit: INFINITY, **)
  super
end

# Named keyword + anonymous keyword rest forwarding
def with_named_and_anonymous_keyword_rest(io, delete: delete_raw?, **)
  super
end

# Anonymous keyword rest on its own
def only_anonymous_keyword_rest(**)
  super
end
