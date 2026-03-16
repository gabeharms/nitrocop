# Block if with multiline condition
if foo &&
   ^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
   bar
  do_something
end

# Block unless with multiline condition
unless foo &&
       ^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
       bar
  do_something
end

# Block while with multiline condition
while foo &&
      ^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
      bar
  do_something
end

# Block until with multiline condition
until foo ||
      ^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
      bar
  do_something
end

# elsif with multiline condition
if condition
  do_something
elsif multiline &&
      ^^^^^^^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
   condition
  do_something_else
end

# Modifier if with multiline condition and right sibling
do_something if multiline &&
                ^^^^^^^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
                condition
do_something_else

# case/when with multiline condition
case x
when foo,
^^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
    bar
  do_something
end

# rescue with multiline exceptions
begin
  do_something
rescue FooError,
^^^^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
  BarError
  handle_error
end

# Modifier while (non-begin form) with multiline condition — always check
nil while
    foo &&
    ^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
    bar
do_something

# Block if with multiline do..end block condition
if items.find do |item|
   ^^^^^^^^^^ Layout/EmptyLineAfterMultilineCondition: Use empty line after multiline condition.
     item.ready?
   end
  process
end
