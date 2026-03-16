foo(bar,
    ^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  baz)

something(first,
          ^^^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  second,
  third)

method_call(arg1,
            ^^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  arg2)

super(bar,
      ^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  baz)

# Block argument (&) on a separate line
client.send(request_method, :post,
            ^^^^^^^^^^^^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
            &@read_body_chunk_block)

# Block argument as only second arg on next line
process(data,
        ^^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
        &handler)

# Multiple args with block argument spanning lines
execute(cmd, env, opts,
        ^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
        &callback)

# Super with block argument on a separate line
super(arg,
      ^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
      &block)

# Block argument as sole argument spanning multiple lines
configure(&configuration_block(
          ^^^^^^^^^^^^^^^^^^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  verifier_email: email,
  verifier_domain: domain
))

# Block argument with hash-to-proc as sole argument
process(&{
        ^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  key_one: :value_one,
  key_two: :value_two
}.to_proc)

# Block argument with lambda as sole argument
endpoint(&-> {
         ^^^^ Layout/FirstMethodArgumentLineBreak: Add a line break before the first argument of a multi-line method call.
  @record
})
