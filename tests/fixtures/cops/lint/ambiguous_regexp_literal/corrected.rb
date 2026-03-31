p (/pattern/)
p (/pattern/), foo
puts line.grep (/pattern/)
p (/pattern/).do_something
p (/pattern/).do_something(42)
p (/pattern/).do_something.do_something
class MyTest
  test '#foo' do
    assert_match (/expected/), actual
  end
end
expect('RuboCop').to(match (/Cop/))
expect('RuboCop').to match (/Robo/)
assert (/some pattern/) =~ some_string
p (/pattern/) do
  p (/pattern/)
end
p (/pattern/), foo do |arg|
end
