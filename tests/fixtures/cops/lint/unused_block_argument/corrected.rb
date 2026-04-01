do_something do |used, _unused|
  puts used
end

do_something do |_bar|
  puts :foo
end

[1, 2, 3].each do |_x|
  puts "hello"
end

-> (_foo, _bar) { do_something }

->(_arg) { 1 }

obj.method { |foo, *_bars, baz| stuff(foo, baz) }

1.times do |index; _block_local_variable|
  puts index
end

define_method(:foo) do |_bar|
  puts :baz
end

-> (_foo, bar) { puts bar }

# Variable shadowing in nested blocks: outer `item` is unused because
# inner block shadows it — the read of `item` inside refers to the inner param
items.each do |_item|
  results.map do |item|
    item.name
  end
end

# Nested lambda shadows outer param
records.each do |_record|
  -> (record) { record.save }
end

# Multiple levels of nesting with shadowing
data.each do |_value|
  items.each do |value|
    value.process
  end
end

# Blocks inside def methods should still detect unused args
def process_items
  items.each do |_item|
    puts "processing"
  end
end

# Block inside class > def
class Worker
  def run
    tasks.each do |_task|
      puts "running"
    end
  end
end

# Nested module > class > def > block
module Services
  class Processor
    def call
      records.map do |_record|
        "done"
      end
    end
  end
end

# Destructured block params: one element unused
translations.find { |(locale, _translation)|
  locale.to_s == I18n.locale.to_s
}

# Destructured params in do..end block
hash.inject([]) do |array, (_id, attributes)|
  array << [attributes[:iso_code]]
end

# Lambda with destructured params
->((_item_id, item_model)) {
  process(item_model: item_model)
}

# Lambda with mixed regular and destructured params
->(_, (_item_id, item_model), _) {
  process(item_model: item_model)
}

# Destructured with splat inside: |(a, *b, c)|
items.each do |(first, *_rest, last)|
  puts first
  puts last
end

# Unused block-pass parameter (&block)
obj.method do |original, env, &_handler|
  original.call(env)
end

# Unused keyword rest parameter (**opts)
->(val, **_opts) { val.to_s }

# Unused keyword rest in block
do_something do |val, **_options|
  puts val
end

# Lambda as default parameter value in method def: unused `row`
def has_many(association_name, model:, foreign_key:, scope: ->(_row) { true })
end

# Stabby lambda as default param with unused `b`
def one_opt_with_stabby(a = -> _b { true }); end

# Lambda in keyword default with unused `node` in args param
def call_node?(node, name:, args: ->(_node) { true })
end
