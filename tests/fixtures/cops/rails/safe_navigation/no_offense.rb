obj&.method_name
obj.send(:method_name)
obj&.foo&.bar
record&.save
user&.name
obj.try(:method_name)
# try/try! with non-symbol first argument — can't convert to safe navigation
linkset.try(short)
obj.try!(method_var)
obj.try!(some_method_name)

# try!/try with operator method symbols — RuboCop skips these because
# the symbol value doesn't match /\w+[=!?]?/
result.try!(:[], "count")
hash.try!(:[], key)
obj.try(:[]=, key, value)
