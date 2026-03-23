while cond do
           ^^ Style/WhileUntilDo: Do not use `do` with multi-line `while`.
end

until cond do
           ^^ Style/WhileUntilDo: Do not use `do` with multi-line `until`.
end

while x.any? do
             ^^ Style/WhileUntilDo: Do not use `do` with multi-line `while`.
  do_something(x.pop)
end

while condition &&
more_condition do
               ^^ Style/WhileUntilDo: Do not use `do` with multi-line `while`.
  body
end

until i > 9 do i += 1
            ^^ Style/WhileUntilDo: Do not use `do` with multi-line `until`.
end

while i < 3 do i += 1
            ^^ Style/WhileUntilDo: Do not use `do` with multi-line `while`.
end

while (((next_sibling = cur_element.next_sibling).name =~ /p|text|div|ul/) || first_p_found.nil?) do
                                                                                                  ^^ Style/WhileUntilDo: Do not use `do` with multi-line `while`.
  cur_element = next_sibling
end
