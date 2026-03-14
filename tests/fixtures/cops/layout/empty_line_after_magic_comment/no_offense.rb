# frozen_string_literal: true

class Foo
end
# no magic comment at all
x = 1

# coding: UTF-8

y = 2

# coding: ISO-8859-15

z = 3

# CoDiNg:   bIg5
$magic_comment_result = __ENCODING__.name
