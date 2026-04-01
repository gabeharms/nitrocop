# Block form: unless ... end with mkdir (2 offenses)
FileUtils.mkdir_p(path)

# Block form: if ... end with remove (2 offenses)
FileUtils.rm_f(path)

# Postfix unless (2 offenses)
FileUtils.mkdir_p(path)

# Postfix if (2 offenses)
FileUtils.rm_f(path)

# Force method makedirs: only existence check offense (1 offense)
FileUtils.makedirs(path)

# Force method rm_f: only existence check offense (1 offense)
FileUtils.rm_f(path)

# Force method rm_rf: only existence check offense (1 offense)
FileUtils.rm_rf(path)

# Negated if with ! (1 offense for force method)
FileUtils.makedirs(path)

# Dir.mkdir with Dir.exist? (2 offenses)
FileUtils.mkdir_p(path)

# Recursive remove methods (2 offenses)
FileUtils.rm_rf(path)

# Fully qualified constant with :: prefix on existence check
FileUtils.rm_f(path)

# Fully qualified constant with :: prefix on file operation
FileUtils.rm_f(path)

# elsif form (only existence check offense, rm_f is force method)
if condition
  do_something
elsif FileTest.exist?(path)
  FileUtils.rm_f(path)
end

# mkdir_p force method (only existence check offense)
FileUtils.mkdir_p(path)

# mkpath force method (only existence check offense)
FileUtils.mkpath(path)

# File.exist? as condition class
FileUtils.rm_f(path)

# Dir.exist? as condition class with rmdir
FileUtils.rm_f(path)

# Shell.exist? as condition class
FileUtils.rm_f(path)

# File.delete with File.exist? postfix if (any const receiver accepted)
FileUtils.rm_f(path)

# File.unlink with File.exist? postfix if
FileUtils.rm_f(path)

# File.delete in block if form
FileUtils.rm_f(path)

# File.unlink with space-separated args (no parens)
FileUtils.rm_f(path)

# Postfix if with File.delete and mismatched quote styles (single vs double)
FileUtils.rm_f('./.slather.yml')

# remove_entry with force: true option (only existence check offense)
FileUtils.remove_entry base_directory, :force => true

# rm with force: true option (only existence check offense)
FileUtils.rm(path, :force => true)

# remove with force: true using new-style hash syntax (only existence check offense)
FileUtils.remove(path, force: true)

# Negated condition using == false
FileUtils.mkdir_p catalogs_path
