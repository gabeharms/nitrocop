# Chain form: File.open(filename).read
File.read(filename)
# Chain form with ::File prefix
File.read(filename)
# Chain form with explicit 'r' mode
File.read(filename)
# Block pass form: &:read
File.read(filename)
# Block pass with 'r' mode
File.read(filename)
# Block form inline
File.read(filename)
# Block form multiline
File.read(filename)
# Binary mode chain
File.binread(filename)
# Binary mode block pass
File.binread(filename)
# Binary mode block form
File.binread(filename)
# r+ mode
File.read(filename)
# rt mode
File.read(filename)
# r+b mode
File.binread(filename)
# r+t mode
File.read(filename)
