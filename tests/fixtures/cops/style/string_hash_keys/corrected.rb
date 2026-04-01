{ :foo => 1 }

{ :bar => 2, :baz => 3 }

x = { :key => 'value' }

# String keys in a hash that is the receiver of gsub (not an argument)
{ :expiration => time, :conditions => conds }.to_json.gsub("\n", "")

# String keys in a hash nested inside an array argument of IO.popen
IO.popen([{:FOO => "bar"}, "ruby", "foo.rb"])

# String keys in a block inside spawn/system (not direct arg)
system("cmd") do
  x = { :inner => 1 }
end

# Non-identifier string keys are also flagged (RuboCop autocorrects to :"Content-Type" etc.)
{ :"Content-Type" => "text/html" }
{ :"foo bar" => 1 }
