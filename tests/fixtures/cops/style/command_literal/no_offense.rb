folders = `find . -type d`.split

result = `ls -la`

output = `echo hello`

name = `whoami`.chomp

path = `pwd`.strip

# %x with inner backticks is allowed in backticks mode
output = %x(echo `ls`)
