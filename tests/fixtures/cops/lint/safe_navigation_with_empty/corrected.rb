# Receiver is a send node (regular method call with `.`) — RuboCop flags these
return unless foo.bar && foo.bar.empty?
bar if collection.find_all && collection.find_all.empty?
do_something if items.select && items.select.empty?

# Receiver is a bare method call (send nil :method) — RuboCop flags these
return unless path && path.empty?
if options && options.empty?
  name
else
  "#<QueueConfiguration #{name} options=#{options.inspect}>"
end
