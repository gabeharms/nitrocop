foo(
  bar,
  baz)

something(
  first,
  second,
  third)

method_call(
  arg1,
  arg2)

super(
  bar,
  baz)

# Block argument (&) on a separate line
client.send(
            request_method, :post,
            &@read_body_chunk_block)

# Block argument as only second arg on next line
process(
        data,
        &handler)

# Multiple args with block argument spanning lines
execute(
        cmd, env, opts,
        &callback)

# Super with block argument on a separate line
super(
      arg,
      &block)

# Block argument as sole argument spanning multiple lines
configure(
          &configuration_block(
  verifier_email: email,
  verifier_domain: domain
))

# Block argument with hash-to-proc as sole argument
process(
        &{
  key_one: :value_one,
  key_two: :value_two
}.to_proc)

# Block argument with lambda as sole argument
endpoint(
         &-> {
  @record
})
