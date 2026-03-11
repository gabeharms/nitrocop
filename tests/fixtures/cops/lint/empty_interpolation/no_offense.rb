"hello #{name}"
"foo #{1 + 2} bar"
"#{x}"
"no interpolation here"
result = "value: #{compute}"
x = "#{a} and #{b}"
ok = "this is the #{1}"
flag = "this is the #{true}"
words = %W[#{''} one two]
more_words = %W[#{nil} one two]
symbols = %I[#{''} one two]
script = "#{<<~TEXT}"
puts :hello
TEXT
