folders = %x(find . -type d).split
          ^^^^^^^^^^^^^^^^^^^ Style/CommandLiteral: Use backticks around command string.

result = %x(ls -la)
         ^^^^^^^^^^ Style/CommandLiteral: Use backticks around command string.

output = %x(echo hello)
         ^^^^^^^^^^^^^^ Style/CommandLiteral: Use backticks around command string.

x = `echo \`ls\``
    ^^^^^^^^^^^^^^ Style/CommandLiteral: Use `%x` around command string.
