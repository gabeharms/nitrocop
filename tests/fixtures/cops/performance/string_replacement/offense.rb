str.gsub('a', 'b')
    ^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub(' ', '-')
    ^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub('x', 'y')
    ^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub!('a', '1')
    ^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr!` instead of `gsub!`.
str.gsub('a', '')
    ^^^^^^^^^^^^^ Performance/StringReplacement: Use `delete` instead of `gsub`.
str.gsub!('a', '')
    ^^^^^^^^^^^^^^ Performance/StringReplacement: Use `delete!` instead of `gsub!`.
str.gsub("Á", "A")
    ^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub("á", "a")
    ^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub("-","")
    ^^^^^^^^^^^^ Performance/StringReplacement: Use `delete` instead of `gsub`.
'a + c'.gsub('+', '-')
        ^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub(/a/, 'b')
    ^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub(/ /, '')
    ^^^^^^^^^^^^^ Performance/StringReplacement: Use `delete` instead of `gsub`.
str.gsub!(/\n/, ',')
    ^^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr!` instead of `gsub!`.
str.gsub(/\t/, ' ')
    ^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub(/\\/, '-')
    ^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub(/\./, '-')
    ^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
str.gsub(/\y/, 'x')
    ^^^^^^^^^^^^^^^^ Performance/StringReplacement: Use `tr` instead of `gsub`.
